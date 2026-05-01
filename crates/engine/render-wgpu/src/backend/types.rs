use amigo_core::AmigoResult;
use amigo_render_api::{RenderBackend, RenderBackendInfo, RenderInitializationReport};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone, Copy)]
pub struct WgpuRenderBackend {
    backends: wgpu::Backends,
}

impl Default for WgpuRenderBackend {
    fn default() -> Self {
        Self {
            backends: wgpu::Backends::all(),
        }
    }
}

impl RenderBackend for WgpuRenderBackend {
    fn info(&self) -> RenderBackendInfo {
        let _ = self.backends;

        RenderBackendInfo {
            backend_name: "wgpu",
            shading_language: "wgsl",
            notes: "Scene-aware early renderer for Amigo playground slices.",
        }
    }

    fn initialize(&self) -> AmigoResult<RenderInitializationReport> {
        let context = self.initialize_headless()?;
        Ok(context.report)
    }
}

pub struct WgpuRenderPlugin;

impl RuntimePlugin for WgpuRenderPlugin {
    fn name(&self) -> &'static str {
        "amigo-render-wgpu"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        let backend = WgpuRenderBackend::default();
        registry.register(backend.info())?;
        registry.register(backend)
    }
}

#[derive(Debug)]
pub struct WgpuHeadlessContext {
    pub report: RenderInitializationReport,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct WgpuSurfaceState {
    pub report: RenderInitializationReport,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
}
