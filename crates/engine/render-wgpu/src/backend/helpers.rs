use amigo_core::AmigoError;
use amigo_core::AmigoResult;
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

pub(crate) fn request_adapter(
    instance: &wgpu::Instance,
    compatible_surface: Option<&wgpu::Surface<'_>>,
) -> AmigoResult<wgpu::Adapter> {
    block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface,
    }))
    .map_err(|error| AmigoError::Message(error.to_string()))
}

pub(crate) fn create_device_descriptor<'a>() -> wgpu::DeviceDescriptor<'a> {
    let mut descriptor = wgpu::DeviceDescriptor::default();
    descriptor.label = Some("amigo-wgpu-device");
    descriptor
}

struct NoopWake;

impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

pub(crate) fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
