use crate::{RenderContribution2d, Renderable2dItem};

pub trait RenderExtractionOutput2d {
    fn push_renderable_2d(&mut self, item: Renderable2dItem);
    fn push_render_contribution_2d(&mut self, contribution: RenderContribution2d);
}

#[derive(Debug, Clone, Default)]
pub struct RenderExtractionOutput2dBuffer {
    pub renderables: Vec<Renderable2dItem>,
    pub contributions: Vec<RenderContribution2d>,
}

impl RenderExtractionOutput2d for RenderExtractionOutput2dBuffer {
    fn push_renderable_2d(&mut self, item: Renderable2dItem) {
        self.renderables.push(item);
    }

    fn push_render_contribution_2d(&mut self, contribution: RenderContribution2d) {
        self.contributions.push(contribution);
    }
}
