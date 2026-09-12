//! DX12 interop is confined to the backend; no CPU pixel access is used.
use super::WgpuOffscreenTarget;
use amigo_playground_native::{NativeGpuResources, NativeHandle, PendingHandles};
use std::mem::ManuallyDrop;
use windows::{
    Win32::{
        Foundation::GENERIC_ALL,
        Graphics::{Direct3D12::*, Dxgi::Common::*},
    },
    core::Interface,
};

struct Slot {
    texture: ID3D12Resource,
    allocator: ID3D12CommandAllocator,
    commands: ID3D12GraphicsCommandList,
    value: u64,
    source: Option<wgpu::Texture>,
}
pub struct WgpuSharedTextures {
    queue: ID3D12CommandQueue,
    ready: ID3D12Fence,
    done: ID3D12Fence,
    slots: Vec<Slot>,
    descriptor: NativeGpuResources,
    pending_handles: PendingHandles,
}

impl WgpuSharedTextures {
    pub fn new(
        target: &WgpuOffscreenTarget,
        generation: u64,
        peer_pid: u32,
    ) -> Result<Self, String> {
        unsafe { Self::create(target, generation, peer_pid).map_err(|e| e.to_string()) }
    }
    unsafe fn create(
        target: &WgpuOffscreenTarget,
        generation: u64,
        peer_pid: u32,
    ) -> windows::core::Result<Self> {
        let hal = unsafe { target.device.as_hal::<wgpu::hal::api::Dx12>() }
            .ok_or_else(windows::core::Error::empty)?;
        let device = hal.raw_device();
        let queue = hal.raw_queue().clone();
        let ready: ID3D12Fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_SHARED)? };
        let done: ID3D12Fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_SHARED)? };
        let mut pending = PendingHandles::new(peer_pid);
        let mut share = |object: &ID3D12DeviceChild| -> windows::core::Result<u64> {
            let handle = NativeHandle(unsafe {
                device.CreateSharedHandle(object, None, GENERIC_ALL.0, None)?
            });
            pending.duplicate(handle.0).map_err(|message| {
                windows::core::Error::new(windows::core::HRESULT(0x80004005u32 as i32), message)
            })
        };
        let mut slots = Vec::new();
        let mut textures = [0u64; 3];
        for handle in &mut textures {
            let mut texture: Option<ID3D12Resource> = None;
            unsafe {
                device.CreateCommittedResource(
                    &D3D12_HEAP_PROPERTIES {
                        Type: D3D12_HEAP_TYPE_DEFAULT,
                        ..Default::default()
                    },
                    D3D12_HEAP_FLAG_SHARED,
                    &D3D12_RESOURCE_DESC {
                        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                        Width: target.width as u64,
                        Height: target.height,
                        DepthOrArraySize: 1,
                        MipLevels: 1,
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                        ..Default::default()
                    },
                    D3D12_RESOURCE_STATE_COMMON,
                    None,
                    &mut texture,
                )?;
            }
            let texture = texture.unwrap();
            *handle = share(&texture.cast()?)?;
            let allocator =
                unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)? };
            let commands: ID3D12GraphicsCommandList = unsafe {
                device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)?
            };
            unsafe {
                commands.Close()?;
            }
            slots.push(Slot {
                texture,
                allocator,
                commands,
                value: 0,
                source: None,
            });
        }
        let luid = unsafe { device.GetAdapterLuid() };
        let descriptor = NativeGpuResources {
            generation,
            size: [target.width, target.height],
            adapter: [luid.LowPart, luid.HighPart as u32],
            textures,
            ready_fence: share(&ready.cast()?)?,
            done_fence: share(&done.cast()?)?,
        };
        Ok(Self {
            queue,
            ready,
            done,
            slots,
            descriptor,
            pending_handles: pending,
        })
    }

    /// The transport must return success only after taking ownership of the
    /// message. Failed/abandoned exports close their duplicates in the peer.
    pub fn export_resources(
        &mut self,
        send: impl FnOnce(NativeGpuResources) -> Result<(), String>,
    ) -> Result<(), String> {
        send(self.descriptor.clone())?;
        self.pending_handles.transferred();
        Ok(())
    }

    pub fn idle(&self) -> bool {
        self.slots.iter().all(|slot| unsafe {
            self.ready.GetCompletedValue() >= slot.value
                && self.done.GetCompletedValue() >= slot.value
        })
    }

    pub fn copy(
        &mut self,
        target: &WgpuOffscreenTarget,
        value: u64,
    ) -> Result<Option<usize>, String> {
        unsafe { self.copy_inner(target, value).map_err(|e| e.to_string()) }
    }
    unsafe fn copy_inner(
        &mut self,
        target: &WgpuOffscreenTarget,
        value: u64,
    ) -> windows::core::Result<Option<usize>> {
        let Some(index) = self.slots.iter().position(|slot| unsafe {
            self.ready.GetCompletedValue() >= slot.value
                && self.done.GetCompletedValue() >= slot.value
        }) else {
            return Ok(None);
        };
        // Tell WGPU about the final source state, so its next submission emits
        // the correct reverse transition. Raw commands preserve COPY_SOURCE.
        let mut encoder = target.device.create_command_encoder(&Default::default());
        encoder.transition_resources(
            std::iter::empty(),
            std::iter::once(wgpu::TextureTransition {
                texture: &target.texture,
                selector: None,
                state: wgpu::TextureUses::COPY_SRC,
            }),
        );
        target.queue.submit([encoder.finish()]);
        let source = unsafe { target.texture.as_hal::<wgpu::hal::api::Dx12>() }
            .ok_or_else(windows::core::Error::empty)?;
        let slot = &mut self.slots[index];
        unsafe {
            slot.allocator.Reset()?;
            slot.commands.Reset(&slot.allocator, None)?;
            barrier(
                &slot.commands,
                &slot.texture,
                D3D12_RESOURCE_STATE_COMMON,
                D3D12_RESOURCE_STATE_COPY_DEST,
            );
            slot.commands
                .CopyResource(&slot.texture, source.raw_resource());
            barrier(
                &slot.commands,
                &slot.texture,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_COMMON,
            );
            slot.commands.Close()?;
            self.queue
                .ExecuteCommandLists(&[Some(slot.commands.cast()?)]);
            self.queue.Signal(&self.ready, value)?;
        }
        slot.value = value;
        slot.source = Some(target.texture.clone());
        Ok(Some(index))
    }
}

impl Drop for WgpuSharedTextures {
    fn drop(&mut self) {
        // Retirement may wait on our own queue, never on a terminated peer.
        let maximum = self.slots.iter().map(|slot| slot.value).max().unwrap_or(0);
        while unsafe { self.ready.GetCompletedValue() } < maximum {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

unsafe fn barrier(
    list: &ID3D12GraphicsCommandList,
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) {
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: ManuallyDrop::new(Some(resource.clone())),
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    };
    unsafe {
        list.ResourceBarrier(&[barrier.clone()]);
        ManuallyDrop::drop(&mut (*barrier.Anonymous.Transition).pResource);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_playground_native::{GpuPresenter, NativeSurface};
    use windows::{Win32::UI::WindowsAndMessaging::*, core::w};
    #[test]
    #[ignore = "requires Windows DX12 adapter and desktop session"]
    fn shared_texture_roundtrip_resize_and_retirement() {
        eprintln!("DX12 roundtrip: creating window");
        let parent = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("Amigo GPU test"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                160,
                120,
                None,
                None,
                None,
                None,
            )
            .unwrap()
        };
        let (send, _) = std::sync::mpsc::channel();
        let surface = NativeSurface::new(parent, send).unwrap();
        let mut target = super::super::WgpuRenderBackend::dx12()
            .initialize_offscreen(64, 32)
            .unwrap();
        eprintln!("DX12 roundtrip: offscreen ready");
        for generation in 1..=3 {
            target.resize(64 * generation as u32, 32 * generation as u32);
            let mut encoder = target.device.create_command_encoder(&Default::default());
            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.2,
                                g: 0.5,
                                b: 0.8,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
            }
            target.queue.submit([encoder.finish()]);
            let mut shared =
                WgpuSharedTextures::new(&target, generation, std::process::id()).unwrap();
            let mut imported = None;
            eprintln!("DX12 roundtrip: importing generation {generation}");
            shared
                .export_resources(|resources| {
                    imported = Some(GpuPresenter::new(surface.hwnd, resources));
                    Ok(()) // The importer owns all received handles even on error.
                })
                .unwrap();
            let mut presenter = imported.unwrap().unwrap();
            eprintln!("DX12 roundtrip: presenting generation {generation}");
            for sequence in 1..=9 {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
                let slot = loop {
                    if let Some(slot) = shared.copy(&target, sequence).unwrap() {
                        break slot;
                    }
                    assert!(
                        std::time::Instant::now() < deadline,
                        "shared texture consumer did not retire its slot"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(1));
                };
                presenter.present(slot, sequence).unwrap();
            }
            drop(presenter);
            eprintln!("DX12 roundtrip: retired generation {generation}");
            assert!(shared.idle());
        }
        drop(surface);
        unsafe {
            DestroyWindow(parent).unwrap();
        }
    }
}
