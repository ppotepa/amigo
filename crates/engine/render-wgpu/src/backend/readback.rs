use super::WgpuOffscreenTarget;
use std::{sync::mpsc, time::Instant};

struct Slot {
    buffer: wgpu::Buffer,
    size: [u32; 2],
    row: u32,
    pending: Option<(
        u64,
        Instant,
        mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    )>,
}

/// Three reusable staging buffers. Polling never waits for GPU completion.
#[derive(Default)]
pub struct WgpuReadbackPool {
    slots: Vec<Slot>,
}

pub struct WgpuReadbackFrame {
    pub ticket: u64,
    pub rgba: Vec<u8>,
    pub milliseconds: f64,
}

impl WgpuReadbackPool {
    pub fn available(&self) -> bool {
        self.slots.len() < 3 || self.slots.iter().any(|slot| slot.pending.is_none())
    }

    pub fn submit(&mut self, target: &WgpuOffscreenTarget, ticket: u64) -> bool {
        let index = match self.slots.iter().position(|slot| slot.pending.is_none()) {
            Some(index) => index,
            None if self.slots.len() < 3 => self.slots.len(),
            None => return false,
        };
        let size = [target.width, target.height];
        let row = (target.width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        if index == self.slots.len() || self.slots[index].size != size {
            let slot = Slot {
                buffer: target.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("playground-readback-slot"),
                    size: u64::from(row) * u64::from(target.height),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }),
                size,
                row,
                pending: None,
            };
            if index == self.slots.len() {
                self.slots.push(slot);
            } else {
                self.slots[index] = slot;
            }
        }
        let slot = &mut self.slots[index];
        let started = Instant::now();
        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("playground-readback-copy"),
            });
        encoder.copy_texture_to_buffer(
            target.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &slot.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(target.height),
                },
            },
            wgpu::Extent3d {
                width: target.width,
                height: target.height,
                depth_or_array_layers: 1,
            },
        );
        target.queue.submit([encoder.finish()]);
        let (send, receive) = mpsc::channel();
        slot.buffer
            .map_async(wgpu::MapMode::Read, .., move |result| {
                let _ = send.send(result);
            });
        slot.pending = Some((ticket, started, receive));
        true
    }

    pub fn poll(&mut self, device: &wgpu::Device) -> Result<Vec<WgpuReadbackFrame>, String> {
        device
            .poll(wgpu::PollType::Poll)
            .map_err(|e| e.to_string())?;
        let mut ready = Vec::new();
        for slot in &mut self.slots {
            let Some((ticket, started, receiver)) = &slot.pending else {
                continue;
            };
            let result = match receiver.try_recv() {
                Ok(result) => result,
                Err(mpsc::TryRecvError::Empty) => continue,
                Err(error) => return Err(error.to_string()),
            };
            result.map_err(|e| e.to_string())?;
            let mapped = slot.buffer.get_mapped_range(..);
            let packed_row = slot.size[0] * 4;
            let rgba = if slot.row == packed_row {
                // All benchmark targets use an aligned packed row. Avoid the
                // per-row bounds/slice work on this hot path.
                mapped.to_vec()
            } else {
                let mut rgba = Vec::with_capacity((slot.size[0] * slot.size[1] * 4) as usize);
                for row in mapped.chunks_exact(slot.row as usize) {
                    rgba.extend_from_slice(&row[..packed_row as usize]);
                }
                rgba
            };
            drop(mapped);
            slot.buffer.unmap();
            ready.push(WgpuReadbackFrame {
                ticket: *ticket,
                rgba,
                milliseconds: started.elapsed().as_secs_f64() * 1000.0,
            });
            slot.pending = None;
        }
        Ok(ready)
    }
}
