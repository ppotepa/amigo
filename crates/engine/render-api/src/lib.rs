use amigo_core::AmigoResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceDescriptor {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

#[derive(Debug, Clone)]
pub struct RenderBackendInfo {
    pub backend_name: &'static str,
    pub shading_language: &'static str,
    pub notes: &'static str,
}

#[derive(Debug, Clone)]
pub struct RenderInitializationReport {
    pub backend_name: &'static str,
    pub adapter_name: String,
    pub adapter_backend: String,
    pub device_type: String,
    pub shader_language: &'static str,
    pub queue_ready: bool,
}

pub trait RenderBackend: Send + Sync {
    fn info(&self) -> RenderBackendInfo;
    fn initialize(&self) -> AmigoResult<RenderInitializationReport>;
}
