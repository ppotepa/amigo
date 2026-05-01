#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiEvents {
    pub on_click: Option<UiEventBinding>,
    pub on_change: Option<UiEventBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiEventBinding {
    pub event: String,
    pub payload: Vec<String>,
}

impl UiEventBinding {
    pub fn new(event: impl Into<String>, payload: Vec<String>) -> Self {
        Self {
            event: event.into(),
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.width && y <= self.y + self.height
    }

    pub fn inset(&self, amount: f32) -> Self {
        let double = amount * 2.0;
        Self {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - double).max(0.0),
            height: (self.height - double).max(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiLayoutNode {
    pub path: String,
    pub rect: UiRect,
    pub node: UiNode,
    pub children: Vec<UiLayoutNode>,
}

