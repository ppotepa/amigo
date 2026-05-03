//! Backend-agnostic rendering contracts used by the app and render backends.
//! It defines initialization reports and extraction traits for frame building.

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

#[derive(Debug, Clone, Default)]
pub struct RenderFramePacket<TOverlay> {
    overlay: Vec<TOverlay>,
}

impl<TOverlay> RenderFramePacket<TOverlay> {
    pub fn new() -> Self {
        Self {
            overlay: Vec::new(),
        }
    }

    pub fn with_overlay(overlay: Vec<TOverlay>) -> Self {
        Self { overlay }
    }

    pub fn push_overlay(&mut self, overlay: TOverlay) {
        self.overlay.push(overlay);
    }

    pub fn extend_overlay<I>(&mut self, overlay: I)
    where
        I: IntoIterator<Item = TOverlay>,
    {
        self.overlay.extend(overlay);
    }

    pub fn overlay(&self) -> &[TOverlay] {
        &self.overlay
    }

    pub fn into_overlay(self) -> Vec<TOverlay> {
        self.overlay
    }
}

pub trait RenderExtractor<TContext, TOverlay>: Send + Sync {
    fn name(&self) -> &'static str;
    fn extract(&self, context: &TContext, packet: &mut RenderFramePacket<TOverlay>);
}

pub struct RenderExtractorRegistry<TContext, TOverlay> {
    extractors: Vec<Box<dyn RenderExtractor<TContext, TOverlay>>>,
}

impl<TContext, TOverlay> RenderExtractorRegistry<TContext, TOverlay> {
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    pub fn register<E>(&mut self, extractor: E)
    where
        E: RenderExtractor<TContext, TOverlay> + 'static,
    {
        self.extractors.push(Box::new(extractor));
    }

    pub fn len(&self) -> usize {
        self.extractors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.extractors.is_empty()
    }

    pub fn extract_all(&self, context: &TContext) -> RenderFramePacket<TOverlay> {
        let mut packet = RenderFramePacket::new();
        for extractor in &self.extractors {
            let _ = extractor.name();
            extractor.extract(context, &mut packet);
        }
        packet
    }
}

impl<TContext, TOverlay> Default for RenderExtractorRegistry<TContext, TOverlay> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait RenderFrameExtractor<TContext, TPacket>: Send + Sync {
    fn name(&self) -> &'static str;
    fn extract(&self, context: &TContext, packet: &mut TPacket);
}

pub struct RenderFrameExtractorRegistry<TContext, TPacket> {
    extractors: Vec<Box<dyn RenderFrameExtractor<TContext, TPacket>>>,
}

impl<TContext, TPacket> RenderFrameExtractorRegistry<TContext, TPacket> {
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    pub fn register<E>(&mut self, extractor: E)
    where
        E: RenderFrameExtractor<TContext, TPacket> + 'static,
    {
        self.extractors.push(Box::new(extractor));
    }

    pub fn len(&self) -> usize {
        self.extractors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.extractors.is_empty()
    }

    pub fn extract_all(&self, context: &TContext) -> TPacket
    where
        TPacket: Default,
    {
        let mut packet = TPacket::default();
        for extractor in &self.extractors {
            let _ = extractor.name();
            extractor.extract(context, &mut packet);
        }
        packet
    }
}

impl<TContext, TPacket> Default for RenderFrameExtractorRegistry<TContext, TPacket> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait RenderBackend: Send + Sync {
    fn info(&self) -> RenderBackendInfo;
    fn initialize(&self) -> AmigoResult<RenderInitializationReport>;
}

#[cfg(test)]
include!("tests.rs");
