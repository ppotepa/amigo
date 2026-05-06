#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorScenePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorFramePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorSceneRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorFrameRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorFrameSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorCamera2D {
    pub center_x: f32,
    pub center_y: f32,
    pub zoom: f32,
}

impl Default for EditorCamera2D {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorCoordinateMapper {
    pub frame: EditorFrameSize,
    pub camera: EditorCamera2D,
}

impl EditorCoordinateMapper {
    pub fn scene_to_frame_point(&self, point: EditorScenePoint) -> EditorFramePoint {
        let zoom = self.camera.zoom.max(0.0001);

        EditorFramePoint {
            x: ((point.x - self.camera.center_x) * zoom) + self.frame.width * 0.5,
            y: self.frame.height * 0.5 - ((point.y - self.camera.center_y) * zoom),
        }
    }

    #[allow(dead_code)]
    pub fn frame_to_scene_point(&self, point: EditorFramePoint) -> EditorScenePoint {
        let zoom = self.camera.zoom.max(0.0001);

        EditorScenePoint {
            x: ((point.x - self.frame.width * 0.5) / zoom) + self.camera.center_x,
            y: ((self.frame.height * 0.5 - point.y) / zoom) + self.camera.center_y,
        }
    }

    pub fn scene_to_frame_rect(&self, rect: EditorSceneRect) -> EditorFrameRect {
        let a = self.scene_to_frame_point(EditorScenePoint {
            x: rect.x,
            y: rect.y,
        });
        let b = self.scene_to_frame_point(EditorScenePoint {
            x: rect.x + rect.width,
            y: rect.y + rect.height,
        });

        EditorFrameRect {
            x: a.x.min(b.x),
            y: a.y.min(b.y),
            width: (a.x - b.x).abs(),
            height: (a.y - b.y).abs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_scene_origin_to_frame_center() {
        let mapper = EditorCoordinateMapper {
            frame: EditorFrameSize {
                width: 1280.0,
                height: 720.0,
            },
            camera: EditorCamera2D::default(),
        };

        let point = mapper.scene_to_frame_point(EditorScenePoint { x: 0.0, y: 0.0 });

        assert_eq!(point.x, 640.0);
        assert_eq!(point.y, 360.0);
    }

    #[test]
    fn maps_scene_rect_around_origin_to_frame_center() {
        let mapper = EditorCoordinateMapper {
            frame: EditorFrameSize {
                width: 1280.0,
                height: 720.0,
            },
            camera: EditorCamera2D::default(),
        };

        let rect = mapper.scene_to_frame_rect(EditorSceneRect {
            x: -50.0,
            y: -50.0,
            width: 100.0,
            height: 100.0,
        });

        assert_eq!(rect.x, 590.0);
        assert_eq!(rect.y, 310.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 100.0);
    }

    #[test]
    fn frame_to_scene_is_inverse_of_scene_to_frame() {
        let mapper = EditorCoordinateMapper {
            frame: EditorFrameSize {
                width: 1280.0,
                height: 720.0,
            },
            camera: EditorCamera2D {
                center_x: 10.0,
                center_y: -20.0,
                zoom: 2.0,
            },
        };

        let scene = EditorScenePoint { x: 123.0, y: 456.0 };
        let frame = mapper.scene_to_frame_point(scene);
        let back = mapper.frame_to_scene_point(frame);

        assert!((scene.x - back.x).abs() < 0.001);
        assert!((scene.y - back.y).abs() < 0.001);
    }
}
