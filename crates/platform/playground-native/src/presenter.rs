use super::{NativeGpuResources, NativeHandle};
use std::mem::ManuallyDrop;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Direct3D::*, Direct3D12::*, Dxgi::Common::*, Dxgi::*},
    },
    core::Interface,
};

struct CopySlot {
    allocator: ID3D12CommandAllocator,
    commands: ID3D12GraphicsCommandList,
    value: u64,
}

pub struct GpuPresenter {
    queue: ID3D12CommandQueue,
    swapchain: IDXGISwapChain3,
    textures: Vec<ID3D12Resource>,
    ready: ID3D12Fence,
    done: ID3D12Fence,
    slots: Vec<CopySlot>,
    pub generation: u64,
}

impl GpuPresenter {
    pub fn new(hwnd: HWND, resources: NativeGpuResources) -> Result<Self, String> {
        unsafe { Self::create(hwnd, resources).map_err(|e| e.to_string()) }
    }
    unsafe fn create(hwnd: HWND, resources: NativeGpuResources) -> windows::core::Result<Self> {
        let handles: Vec<_> = resources
            .textures
            .into_iter()
            .chain([resources.ready_fence, resources.done_fence])
            .map(|value| NativeHandle(HANDLE(value as usize as *mut _)))
            .collect();
        let factory: IDXGIFactory4 = unsafe { CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))? };
        let adapter: IDXGIAdapter = unsafe {
            factory.EnumAdapterByLuid(LUID {
                LowPart: resources.adapter[0],
                HighPart: resources.adapter[1] as i32,
            })?
        };
        let mut device: Option<ID3D12Device> = None;
        unsafe {
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device)?;
        }
        let device = device.unwrap();
        let queue: ID3D12CommandQueue = unsafe {
            device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                ..Default::default()
            })?
        };
        let mut textures = Vec::new();
        for handle in &handles[..3] {
            let mut texture: Option<ID3D12Resource> = None;
            unsafe {
                device.OpenSharedHandle(handle.0, &mut texture)?;
            }
            textures.push(texture.unwrap());
        }
        let mut ready = None;
        let mut done = None;
        unsafe {
            device.OpenSharedHandle(handles[3].0, &mut ready)?;
            device.OpenSharedHandle(handles[4].0, &mut done)?;
        }
        let swapchain: IDXGISwapChain3 = unsafe {
            factory
                .CreateSwapChainForHwnd(
                    &queue,
                    hwnd,
                    &DXGI_SWAP_CHAIN_DESC1 {
                        Width: resources.size[0],
                        Height: resources.size[1],
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                        BufferCount: 3,
                        Scaling: DXGI_SCALING_STRETCH,
                        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                        AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                        ..Default::default()
                    },
                    None,
                    None,
                )?
                .cast()?
        };
        let mut slots = Vec::new();
        for _ in 0..3 {
            let allocator =
                unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)? };
            let commands: ID3D12GraphicsCommandList = unsafe {
                device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)?
            };
            unsafe {
                commands.Close()?;
            }
            slots.push(CopySlot {
                allocator,
                commands,
                value: 0,
            });
        }
        Ok(Self {
            queue,
            swapchain,
            textures,
            ready: ready.unwrap(),
            done: done.unwrap(),
            slots,
            generation: resources.generation,
        })
    }

    pub fn present(&mut self, slot: usize, value: u64) -> Result<(), String> {
        unsafe { self.present_inner(slot, value).map_err(|e| e.to_string()) }
    }
    unsafe fn present_inner(&mut self, slot: usize, value: u64) -> windows::core::Result<()> {
        if slot >= self.textures.len() {
            return Err(windows::core::Error::empty());
        }
        let index = unsafe { self.swapchain.GetCurrentBackBufferIndex() } as usize;
        let copy = &mut self.slots[index];
        // Only allocator retirement can wait. Producer/consumer texture readiness
        // is synchronized by GPU queues, never by a JavaScript acknowledgment.
        while unsafe { self.done.GetCompletedValue() } < copy.value {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let backbuffer: ID3D12Resource = unsafe { self.swapchain.GetBuffer(index as u32)? };
        unsafe {
            copy.allocator.Reset()?;
            copy.commands.Reset(&copy.allocator, None)?;
            barrier(
                &copy.commands,
                &self.textures[slot],
                D3D12_RESOURCE_STATE_COMMON,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
            );
            barrier(
                &copy.commands,
                &backbuffer,
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_COPY_DEST,
            );
            copy.commands
                .CopyResource(&backbuffer, &self.textures[slot]);
            barrier(
                &copy.commands,
                &self.textures[slot],
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                D3D12_RESOURCE_STATE_COMMON,
            );
            barrier(
                &copy.commands,
                &backbuffer,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_PRESENT,
            );
            copy.commands.Close()?;
            self.queue.Wait(&self.ready, value)?;
            self.queue
                .ExecuteCommandLists(&[Some(copy.commands.cast()?)]);
            let presented = self.swapchain.Present(0, DXGI_PRESENT(0));
            self.queue.Signal(&self.done, value)?;
            copy.value = value;
            presented.ok()?;
        }
        Ok(())
    }
}
impl Drop for GpuPresenter {
    fn drop(&mut self) {
        let maximum = self.slots.iter().map(|slot| slot.value).max().unwrap_or(0);
        while unsafe { self.done.GetCompletedValue() } < maximum {
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
