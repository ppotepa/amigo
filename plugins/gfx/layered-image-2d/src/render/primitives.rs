use amigo_render_api::{
    LayeredImage2dPrimitive, LayeredImageBlendMode2dPrimitive,
    LayeredImageLayerOverride2dPrimitive, LayeredImageViewportFit2dPrimitive, RenderPrimitive2d,
    Renderable2dCommon, Renderable2dItem, Renderable2dKind, VisualMaps2dPrimitive,
};

use crate::layered_image::{
    LayeredImageBlendMode2d, LayeredImageDrawCommand, LayeredImageViewportFit2d,
};

fn blend_mode(mode: LayeredImageBlendMode2d) -> LayeredImageBlendMode2dPrimitive {
    match mode {
        LayeredImageBlendMode2d::Alpha => LayeredImageBlendMode2dPrimitive::Alpha,
        LayeredImageBlendMode2d::Additive => LayeredImageBlendMode2dPrimitive::Additive,
        LayeredImageBlendMode2d::Screen => LayeredImageBlendMode2dPrimitive::Screen,
        LayeredImageBlendMode2d::Multiply => LayeredImageBlendMode2dPrimitive::Multiply,
        LayeredImageBlendMode2d::Lighten => LayeredImageBlendMode2dPrimitive::Lighten,
    }
}

fn viewport_fit(mode: LayeredImageViewportFit2d) -> LayeredImageViewportFit2dPrimitive {
    match mode {
        LayeredImageViewportFit2d::Fixed => LayeredImageViewportFit2dPrimitive::Fixed,
        LayeredImageViewportFit2d::Stretch => LayeredImageViewportFit2dPrimitive::Stretch,
        LayeredImageViewportFit2d::Contain => LayeredImageViewportFit2dPrimitive::Contain,
        LayeredImageViewportFit2d::Cover => LayeredImageViewportFit2dPrimitive::Cover,
    }
}

fn visual_maps_to_primitive(maps: &amigo_scene::VisualMaps2dSceneCommand) -> VisualMaps2dPrimitive {
    VisualMaps2dPrimitive {
        normal: maps.normal.clone(),
        wetness: maps.wetness.clone(),
        emissive: maps.emissive.clone(),
        highlight: maps.highlight.clone(),
    }
}

pub fn layered_image_draw_command_to_render_primitive(
    command: &LayeredImageDrawCommand,
) -> RenderPrimitive2d {
    RenderPrimitive2d::LayeredTexturedQuads(LayeredImage2dPrimitive {
        asset: command.image.asset.clone(),
        size: command.image.size,
        base_opacity: command.image.base_opacity,
        viewport_fit: viewport_fit(command.image.viewport_fit),
        transform: command.transform,
        visual_maps: command.image.visual_maps.as_ref().map(visual_maps_to_primitive),
        layer_overrides: command
            .image
            .layer_overrides
            .iter()
            .map(|layer| LayeredImageLayerOverride2dPrimitive {
                id: layer.id.clone(),
                opacity: layer.opacity,
                enabled: layer.enabled,
                blend_mode: layer.blend_mode.map(blend_mode),
                visual_maps: layer.visual_maps.as_ref().map(visual_maps_to_primitive),
            })
            .collect(),
    })
}

pub fn layered_image_draw_command_to_renderable_2d(
    command: &LayeredImageDrawCommand,
) -> Renderable2dItem {
    Renderable2dItem::new(
        Renderable2dCommon::world(
            command.entity_name.clone(),
            "LayeredImage2D",
            command.render_layer.clone(),
            command.z_index,
            Renderable2dKind::LayeredImage,
        ),
        layered_image_draw_command_to_render_primitive(command),
    )
}
