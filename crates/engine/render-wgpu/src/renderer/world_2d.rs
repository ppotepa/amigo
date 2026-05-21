mod beacon;
mod sprite;
mod text;
mod textured;
mod ui;
mod vector;

pub(crate) use beacon::append_beacon_vfx_primitive_vertices;
pub(crate) use sprite::append_textured_quad_debug_vertices;
pub(crate) use text::append_text_2d_vertices;
pub(crate) use textured::append_textured_tilemap_vertices;
pub(crate) use textured::append_tilemap_primitive_fallback_vertices;
pub(crate) use textured::{
    append_textured_sprite_vertices, append_tinted_textured_sprite_vertices,
};
pub(crate) use ui::append_ui_overlay_vertices;
pub(crate) use vector::{append_vector_primitive_vertices, vector_primitive_viewport_fit_transform};
