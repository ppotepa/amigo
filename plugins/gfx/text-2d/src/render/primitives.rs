use amigo_material_api::MaterialCoverageKind2d;
use amigo_render_api::{
    GlyphRun2dBlendMode, GlyphRun2dGlow, GlyphRun2dOutline, GlyphRun2dPrimitive,
    GlyphRun2dShadow, RenderMaterialBinding2d, RenderPrimitive2d, Renderable2dCommon,
    Renderable2dItem, Renderable2dKind,
};

use crate::{Text2dBlendMode, Text2dDrawCommand};

fn glyph_blend_mode(mode: Text2dBlendMode) -> GlyphRun2dBlendMode {
    match mode {
        Text2dBlendMode::Alpha => GlyphRun2dBlendMode::Alpha,
        Text2dBlendMode::Additive => GlyphRun2dBlendMode::Additive,
        Text2dBlendMode::Multiply => GlyphRun2dBlendMode::Multiply,
        Text2dBlendMode::Screen => GlyphRun2dBlendMode::Screen,
    }
}

pub fn text_draw_command_to_render_primitive(command: &Text2dDrawCommand) -> RenderPrimitive2d {
    let mut color = command.text.style.color;
    color.a = (color.a * command.text.style.opacity).clamp(0.0, 1.0);
    RenderPrimitive2d::GlyphRun(GlyphRun2dPrimitive {
        font: command.text.font.clone(),
        text: command.text.content.clone(),
        bounds: command.text.bounds,
        transform: command.text.transform,
        color,
        font_size: command.text.style.font_size,
        blend: glyph_blend_mode(command.text.style.blend),
        shadow: command.text.style.shadow.map(|shadow| GlyphRun2dShadow {
            color: shadow.color,
            offset: shadow.offset,
        }),
        outline: command.text.style.outline.map(|outline| GlyphRun2dOutline {
            color: outline.color,
            width: outline.width,
        }),
        glow: command.text.style.glow.map(|glow| GlyphRun2dGlow {
            color: glow.color,
            radius: glow.radius,
            intensity: glow.intensity,
            passes: glow.passes,
        }),
        material: RenderMaterialBinding2d::new(
            command.material,
            command.render_contributions.clone(),
            MaterialCoverageKind2d::Glyphs,
        ),
    })
}

pub fn text_draw_command_to_renderable_2d(command: &Text2dDrawCommand) -> Renderable2dItem {
    Renderable2dItem::new(
        Renderable2dCommon::world(
            command.entity_name.clone(),
            "Text2D",
            command.render_layer.clone(),
            command.z_index,
            Renderable2dKind::Text,
        ),
        text_draw_command_to_render_primitive(command),
    )
}
