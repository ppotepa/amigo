use amigo_playground_api::*;
use amigo_playground_native::*;
use amigo_render_wgpu::{WgpuOffscreenTarget, WgpuSharedTextures};
use std::{collections::BTreeMap, sync::Arc};

pub(super) struct NativeProducer {
    link: Arc<NativeLink>,
    key: Option<(PlaygroundViewportMode, u64, [u32; 2])>,
    rgba: Vec<RgbaMapping>,
    gpu: Option<WgpuSharedTextures>,
    ready: bool,
    failed: bool,
    in_flight: BTreeMap<u64, (u64, usize, Arc<PlaygroundFrameQueue>)>,
}
impl NativeProducer {
    pub fn new(link: Arc<NativeLink>) -> Self {
        Self {
            link,
            key: None,
            rgba: Vec::new(),
            gpu: None,
            ready: false,
            failed: false,
            in_flight: BTreeMap::new(),
        }
    }
    pub fn retire_failed(&mut self) {
        self.failed = true;
        self.ready = false;
        for (sequence, (generation, _, frames)) in std::mem::take(&mut self.in_flight) {
            frames.acknowledge(&PlaygroundFrameAck {
                sequence,
                generation,
                presented: false,
                decode_ms: 0.0,
                present_ms: 0.0,
            });
        }
        self.gpu = None;
        self.rgba.clear();
    }
    pub fn generation(&self) -> u64 {
        self.key.map_or(0, |(_, generation, _)| generation)
    }
    pub fn poll(&mut self) -> Result<(), String> {
        let mut failure = None;
        for message in self.link.drain() {
            match message {
                NativeMessage::RgbaBuffers {
                    generation,
                    size,
                    handles,
                } => {
                    // Own every received handle before any fallible mapping.
                    let handles = handles
                        .into_iter()
                        .map(|handle| unsafe { NativeHandle::from_raw(handle) })
                        .collect::<Vec<_>>();
                    let mappings = handles
                        .into_iter()
                        .map(|handle| {
                            RgbaMapping::from_handle(
                                handle,
                                size[0] as usize * size[1] as usize * 4,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>();
                    let mappings = match mappings {
                        Ok(mappings) => mappings,
                        Err(error) => {
                            self.retire_failed();
                            failure = Some(error);
                            continue;
                        }
                    };
                    if self.key == Some((PlaygroundViewportMode::LocalRgba, generation, size)) {
                        self.rgba = mappings;
                        self.ready = true;
                    }
                }
                NativeMessage::GpuReady { generation } => {
                    if self.key.is_some_and(|(mode, current, _)| {
                        mode == PlaygroundViewportMode::NativeGpu && current == generation
                    }) {
                        self.ready = true;
                    }
                }
                NativeMessage::Ack { ack } => {
                    if self
                        .in_flight
                        .get(&ack.sequence)
                        .is_some_and(|(generation, _, _)| *generation == ack.generation)
                    {
                        let (_, _, frames) = self.in_flight.remove(&ack.sequence).unwrap();
                        frames.acknowledge(&ack);
                    }
                }
                NativeMessage::Error {
                    generation,
                    message,
                } => {
                    if generation == 0
                        || self
                            .key
                            .is_some_and(|(_, current, _)| current == generation)
                    {
                        self.retire_failed();
                        failure = Some(message);
                    }
                }
                _ => failure = Some("unexpected native companion message".into()),
            }
        }
        failure.map_or(Ok(()), Err)
    }
    pub fn prepare(
        &mut self,
        mode: PlaygroundViewportMode,
        generation: u64,
        target: &WgpuOffscreenTarget,
    ) -> Result<bool, String> {
        let key = (mode, generation, [target.width, target.height]);
        if self.key != Some(key) {
            // Resize can overtake resource negotiation. Finish one handshake
            // before configuring the newest requested generation; never queue
            // a sequence of abandoned swapchains/shared-buffer sets.
            if self.key.is_some() && !self.ready && !self.failed {
                return Ok(false);
            }
            // Retire a generation only after its consumer is finished. Backends
            // additionally check the GPU fence before releasing texture slots.
            if !self.in_flight.is_empty() || self.gpu.as_ref().is_some_and(|gpu| !gpu.idle()) {
                return Ok(false);
            }
            self.key = Some(key);
            self.ready = false;
            self.failed = true;
            self.rgba.clear();
            self.gpu = None;
            match mode {
                PlaygroundViewportMode::LocalRgba => {
                    self.link.send(NativeMessage::RgbaConfigure {
                        generation,
                        size: key.2,
                    })?
                }
                PlaygroundViewportMode::NativeGpu => {
                    let mut gpu = WgpuSharedTextures::new(
                        target,
                        generation,
                        self.link
                            .peer_pid
                            .load(std::sync::atomic::Ordering::Acquire),
                    )?;
                    gpu.export_resources(|resources| {
                        self.link.send(NativeMessage::GpuConfigure { resources })
                    })?;
                    self.gpu = Some(gpu);
                }
                PlaygroundViewportMode::Jpeg => return Ok(true),
            }
            self.failed = false;
        }
        Ok(self.ready && !self.failed)
    }
    pub fn present_gpu(
        &mut self,
        target: &WgpuOffscreenTarget,
        header: PlaygroundFrameHeader,
        permit: PlaygroundFramePermit,
        frames: Arc<PlaygroundFrameQueue>,
    ) -> Result<bool, String> {
        let Some(slot) = self
            .gpu
            .as_mut()
            .ok_or("native textures not configured")?
            .copy(target, header.sequence)?
        else {
            return Ok(false);
        };
        self.send(slot, header.sequence, header, permit, frames)?;
        Ok(true)
    }
    pub fn present_rgba(
        &mut self,
        rgba: &[u8],
        header: PlaygroundFrameHeader,
        permit: PlaygroundFramePermit,
        frames: Arc<PlaygroundFrameQueue>,
    ) -> Result<(), String> {
        if self.key
            != Some((
                PlaygroundViewportMode::LocalRgba,
                header.generation,
                header.size,
            ))
        {
            return Ok(());
        }
        let Some(slot) = (0..self.rgba.len()).find(|slot| {
            !self
                .in_flight
                .values()
                .any(|(_, occupied, _)| occupied == slot)
        }) else {
            return Ok(());
        };
        self.rgba[slot].write(rgba)?;
        self.send(slot, 0, header, permit, frames)
    }
    fn send(
        &mut self,
        slot: usize,
        fence: u64,
        header: PlaygroundFrameHeader,
        permit: PlaygroundFramePermit,
        frames: Arc<PlaygroundFrameQueue>,
    ) -> Result<(), String> {
        let sequence = header.sequence;
        let generation = header.generation;
        self.link.send(NativeMessage::Frame {
            slot,
            fence,
            header,
        })?;
        self.in_flight.insert(sequence, (generation, slot, frames));
        permit.handoff();
        Ok(())
    }
}
