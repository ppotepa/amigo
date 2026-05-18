use crate::renderer::*;

pub(super) fn emergency_overlay_lines(
    request_lines: &[WgpuEmergencyOverlayLine],
    renderer_lines: &[WgpuEmergencyOverlayLine],
) -> Vec<WgpuEmergencyOverlayLine> {
    request_lines
        .iter()
        .chain(renderer_lines.iter())
        .rev()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

pub(super) fn push_emergency_overlay_line(
    lines: &mut Vec<WgpuEmergencyOverlayLine>,
    level: WgpuEmergencyOverlayLevel,
    message: String,
) {
    if lines
        .last()
        .is_some_and(|line| line.level == level && line.message == message)
    {
        return;
    }

    lines.push(WgpuEmergencyOverlayLine { level, message });
    if lines.len() > 5 {
        lines.remove(0);
    }
}

pub(super) fn emergency_overlay_color_batches(
    viewport: &Viewport,
    target_width: u32,
    lines: &[WgpuEmergencyOverlayLine],
) -> Vec<ColorBatch> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut vertices = Vec::new();
    let pixel = 2.0;
    let row_height = 18.0;
    let x = 12.0;
    let y = 12.0;
    let width = (target_width as f32 - x * 2.0).max(80.0);
    let height = row_height * lines.len() as f32 + 10.0;
    append_screen_rect(
        &mut vertices,
        viewport,
        x - 6.0,
        y - 6.0,
        width,
        height,
        ColorRgba::new(0.0, 0.0, 0.0, 0.72),
    );

    for (index, line) in lines.iter().enumerate() {
        let line_y = y + index as f32 * row_height;
        let label = match line.level {
            WgpuEmergencyOverlayLevel::Warning => "WARNING",
            WgpuEmergencyOverlayLevel::Error => "ERROR",
        };
        let color = match line.level {
            WgpuEmergencyOverlayLevel::Warning => ColorRgba::new(1.0, 0.84, 0.20, 1.0),
            WgpuEmergencyOverlayLevel::Error => ColorRgba::new(1.0, 0.22, 0.18, 1.0),
        };
        let text = format!("{label}: {}", line.message);
        append_bitmap_text(
            &mut vertices,
            viewport,
            x,
            line_y,
            pixel,
            &text,
            color,
            width - 12.0,
        );
    }

    vec![ColorBatch {
        blend_mode: ParticleBlendMode2d::Alpha,
        vertices,
    }]
}

fn append_screen_rect(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: ColorRgba,
) {
    let points = [
        ndc_from_ui_screen(Vec2::new(x, y), viewport),
        ndc_from_ui_screen(Vec2::new(x + width, y), viewport),
        ndc_from_ui_screen(Vec2::new(x + width, y + height), viewport),
        ndc_from_ui_screen(Vec2::new(x, y + height), viewport),
    ];
    push_quad(vertices, points[0], points[1], points[2], points[3], color);
}

fn append_bitmap_text(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    x: f32,
    y: f32,
    pixel: f32,
    text: &str,
    color: ColorRgba,
    max_width: f32,
) {
    let mut cursor_x = x;
    for ch in text.chars().flat_map(char::to_uppercase) {
        if cursor_x + 6.0 * pixel > x + max_width {
            break;
        }
        append_bitmap_glyph(vertices, viewport, cursor_x, y, pixel, ch, color);
        cursor_x += 6.0 * pixel;
    }
}

fn append_bitmap_glyph(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    x: f32,
    y: f32,
    pixel: f32,
    ch: char,
    color: ColorRgba,
) {
    for (row, bits) in bitmap_glyph(ch).iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) == 0 {
                continue;
            }
            append_screen_rect(
                vertices,
                viewport,
                x + col as f32 * pixel,
                y + row as f32 * pixel,
                pixel,
                pixel,
                color,
            );
        }
    }
}

fn bitmap_glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x12, 0x12, 0x0C],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x10, 0x1E, 0x01, 0x01, 0x1E],
        '6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x01, 0x0E],
        ':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x04, 0x04, 0x08],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '\\' => [0x10, 0x10, 0x08, 0x04, 0x02, 0x01, 0x01],
        ' ' => [0x00; 7],
        _ => [0x1F, 0x01, 0x02, 0x04, 0x04, 0x00, 0x04],
    }
}

impl WgpuSceneRenderer {
    pub(crate) fn record_emergency_error(&mut self, message: impl Into<String>) {
        push_emergency_overlay_line(
            &mut self.emergency_overlay_lines,
            WgpuEmergencyOverlayLevel::Error,
            message.into(),
        );
    }

    pub(super) fn emergency_overlay_color_batch_for_surface(
        &self,
        surface: &WgpuSurfaceState,
        lines: &[WgpuEmergencyOverlayLine],
    ) -> Vec<ColorBatch> {
        let viewport = Viewport::from_surface(surface);
        emergency_overlay_color_batches(&viewport, surface.config.width, lines)
    }

    pub(super) fn render_emergency_overlay_to_offscreen(
        &self,
        target: &mut WgpuOffscreenTarget,
        lines: &[WgpuEmergencyOverlayLine],
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_offscreen(target);
        let color_batches = emergency_overlay_color_batches(&viewport, target.width, lines);
        if color_batches.iter().all(|batch| batch.vertices.is_empty()) {
            return Ok(());
        }

        let color_vertex_buffers = color_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-emergency-overlay-vertices"),
                        contents: vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-emergency-overlay-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-emergency-overlay-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in color_batches.iter().enumerate() {
                pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(0, color_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }
}
