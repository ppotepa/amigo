use amigo_render_wgpu::UiViewportSize;

use crate::state::EditorRect;

pub const TOP_H: f32 = 34.0;
pub const BOTTOM_H: f32 = 30.0;
pub const LEFT_W: f32 = 330.0;
pub const RIGHT_W: f32 = 360.0;
pub const PAD: f32 = 8.0;
pub const ROW_H: f32 = 22.0;
pub const ROW_GAP: f32 = 4.0;
pub(crate) const INSPECTOR_DOCK_W: f32 = 380.0;
pub(crate) const INSPECTOR_DOCK_MARGIN: f32 = 12.0;
pub(crate) const INSPECTOR_DOCK_TOP: f32 = 48.0;
pub(crate) const INSPECTOR_DOCK_BOTTOM: f32 = 48.0;
pub const GAME_VIEWPORT_LOGICAL_W: f32 = 1280.0;
pub const GAME_VIEWPORT_LOGICAL_H: f32 = 720.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorLayout {
    pub viewport: UiViewportSize,
    pub left_panel: EditorPanelLayout,
    pub center_panel: EditorPanelLayout,
    pub right_panel: EditorPanelLayout,
    pub top_bar: EditorPanelLayout,
    pub bottom_bar: EditorPanelLayout,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorPanelLayout {
    pub rect: EditorRect,
    pub content_rect: EditorRect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorScrollLayout {
    pub visible_top: f32,
    pub visible_bottom: f32,
    pub virtual_y: f32,
    pub render_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorGameViewportLayout {
    pub rect: EditorRect,
    pub logical_width: f32,
    pub logical_height: f32,
    pub scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorPanelKind {
    None,
    TopBar,
    Tree,
    Viewport,
    Properties,
    BottomBar,
}

impl EditorLayout {
    pub fn new(viewport: UiViewportSize) -> Self {
        let (left_w, right_w, center_w) = panel_widths(viewport.width);
        let body_h = body_height(viewport);

        let top_bar = panel_layout(8.0, 6.0, viewport.width - 16.0, TOP_H);
        let bottom_bar = panel_layout(
            8.0,
            viewport.height - BOTTOM_H - 6.0,
            viewport.width - 16.0,
            BOTTOM_H,
        );
        let left_panel = panel_layout(8.0, TOP_H + 8.0, left_w, body_h);
        let center_panel = panel_layout(left_w + 16.0, TOP_H + 8.0, center_w, body_h);
        let right_panel =
            panel_layout(viewport.width - right_w - 8.0, TOP_H + 8.0, right_w, body_h);

        Self {
            viewport,
            left_panel,
            center_panel,
            right_panel,
            top_bar,
            bottom_bar,
        }
    }

    pub fn tree_scroll_layout(self, scroll: f32) -> EditorScrollLayout {
        scroll_layout(self.left_panel, ROW_H * 2.0, scroll)
    }

    pub fn properties_scroll_layout(self, scroll: f32) -> EditorScrollLayout {
        scroll_layout(self.right_panel, ROW_H, scroll)
    }

    pub fn panel_for_point(self, x: f32, y: f32) -> EditorPanelKind {
        if self.left_panel.rect.contains(x, y) {
            EditorPanelKind::Tree
        } else if self.right_panel.rect.contains(x, y) {
            EditorPanelKind::Properties
        } else if self.center_panel.rect.contains(x, y) {
            EditorPanelKind::Viewport
        } else if self.top_bar.rect.contains(x, y) {
            EditorPanelKind::TopBar
        } else if self.bottom_bar.rect.contains(x, y) {
            EditorPanelKind::BottomBar
        } else {
            EditorPanelKind::None
        }
    }

    pub fn tree_row_rect(self, depth: usize, render_y: f32) -> EditorRect {
        let indent = depth as f32 * 14.0;
        EditorRect {
            x: self.left_panel.content_rect.x + indent,
            y: render_y,
            width: (self.left_panel.content_rect.width - indent).max(0.0),
            height: ROW_H,
        }
    }

    pub fn property_row_rect(self, render_y: f32) -> EditorRect {
        EditorRect {
            x: self.right_panel.content_rect.x,
            y: render_y,
            width: self.right_panel.content_rect.width,
            height: ROW_H,
        }
    }

    pub fn game_viewport_rect(self) -> EditorRect {
        self.game_viewport_layout().rect
    }

    pub fn game_viewport_layout(self) -> EditorGameViewportLayout {
        let rect = fitted_rect(
            self.center_panel.content_rect,
            GAME_VIEWPORT_LOGICAL_W,
            GAME_VIEWPORT_LOGICAL_H,
        );
        EditorGameViewportLayout {
            rect,
            logical_width: GAME_VIEWPORT_LOGICAL_W,
            logical_height: GAME_VIEWPORT_LOGICAL_H,
            scale: (rect.width / GAME_VIEWPORT_LOGICAL_W)
                .min(rect.height / GAME_VIEWPORT_LOGICAL_H),
        }
    }
}

impl EditorGameViewportLayout {
    pub fn screen_to_logical(self, x: f32, y: f32) -> Option<(f32, f32)> {
        self.screen_to_logical_with_view(x, y, 0.0, 0.0, 1.0)
    }

    pub fn screen_to_logical_with_view(
        self,
        x: f32,
        y: f32,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) -> Option<(f32, f32)> {
        if !self.rect.contains(x, y) || self.scale <= 0.0 {
            return None;
        }

        let zoom = zoom.max(0.01);
        Some((
            (x - self.rect.x - pan_x) / (self.scale * zoom),
            (y - self.rect.y - pan_y) / (self.scale * zoom),
        ))
    }

    pub fn logical_rect_to_screen(self, rect: EditorRect) -> EditorRect {
        self.logical_rect_to_screen_with_view(rect, 0.0, 0.0, 1.0)
    }

    pub fn logical_rect_to_screen_with_view(
        self,
        rect: EditorRect,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) -> EditorRect {
        let zoom = zoom.max(0.01);
        EditorRect {
            x: self.rect.x + pan_x + rect.x * self.scale * zoom,
            y: self.rect.y + pan_y + rect.y * self.scale * zoom,
            width: rect.width * self.scale * zoom,
            height: rect.height * self.scale * zoom,
        }
    }
}

impl EditorScrollLayout {
    pub fn is_visible(self) -> bool {
        self.virtual_y >= self.visible_top && self.virtual_y <= self.visible_bottom
    }

    pub fn advance_virtual(&mut self) {
        self.virtual_y += ROW_H + ROW_GAP;
    }

    pub fn advance_rendered(&mut self) {
        self.virtual_y += ROW_H + ROW_GAP;
        self.render_y += ROW_H + ROW_GAP;
    }
}

pub fn panel_widths(viewport_width: f32) -> (f32, f32, f32) {
    let available = (viewport_width - 32.0).max(0.0);
    if available >= LEFT_W + RIGHT_W + 240.0 {
        let center = available - LEFT_W - RIGHT_W;
        (LEFT_W, RIGHT_W, center)
    } else {
        let side = if available >= 420.0 {
            (available * 0.34).clamp(180.0, LEFT_W.min(available * 0.5))
        } else {
            available * 0.42
        };
        let right = side.min(RIGHT_W);
        let left = side.min(LEFT_W);
        let center = (available - left - right).max(0.0);
        (left, right, center)
    }
}

fn body_height(viewport: UiViewportSize) -> f32 {
    (viewport.height - TOP_H - BOTTOM_H - 16.0).max(0.0)
}

pub(crate) fn panel_layout(x: f32, y: f32, width: f32, height: f32) -> EditorPanelLayout {
    let rect = EditorRect {
        x,
        y,
        width: width.max(0.0),
        height: height.max(0.0),
    };
    let content_rect = EditorRect {
        x: rect.x + PAD,
        y: rect.y + PAD,
        width: (rect.width - PAD * 2.0).max(0.0),
        height: (rect.height - PAD * 2.0).max(0.0),
    };
    EditorPanelLayout { rect, content_rect }
}

fn fitted_rect(bounds: EditorRect, logical_width: f32, logical_height: f32) -> EditorRect {
    let aspect = logical_width / logical_height.max(1.0);
    let mut width = bounds.width;
    let mut height = width / aspect;

    if height > bounds.height {
        height = bounds.height;
        width = height * aspect;
    }

    EditorRect {
        x: bounds.x + (bounds.width - width) * 0.5,
        y: bounds.y + (bounds.height - height) * 0.5,
        width,
        height,
    }
}

fn scroll_layout(panel: EditorPanelLayout, header_height: f32, scroll: f32) -> EditorScrollLayout {
    let visible_top = panel.content_rect.y + header_height;
    let visible_bottom = panel.content_rect.y + panel.content_rect.height;
    EditorScrollLayout {
        visible_top,
        visible_bottom,
        virtual_y: visible_top - scroll.max(0.0),
        render_y: visible_top,
    }
}

pub(crate) fn inspector_dock_panel(viewport: UiViewportSize) -> EditorPanelLayout {
    let x = (viewport.width - INSPECTOR_DOCK_W - INSPECTOR_DOCK_MARGIN).max(INSPECTOR_DOCK_MARGIN);
    let y = INSPECTOR_DOCK_TOP;
    let h = (viewport.height - INSPECTOR_DOCK_TOP - INSPECTOR_DOCK_BOTTOM).max(160.0);
    panel_layout(x, y, INSPECTOR_DOCK_W, h)
}

pub(crate) fn property_row_rect_for_panel(
    panel: &EditorPanelLayout,
    y: f32,
    height: f32,
) -> EditorRect {
    EditorRect {
        x: panel.content_rect.x,
        y,
        width: panel.content_rect.width,
        height,
    }
}

pub(crate) fn properties_scroll_layout_for_panel(
    panel: &EditorPanelLayout,
    scroll: f32,
) -> EditorScrollLayout {
    scroll_layout(*panel, ROW_H, scroll)
}
