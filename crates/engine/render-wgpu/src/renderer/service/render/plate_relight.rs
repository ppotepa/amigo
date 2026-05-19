use std::{mem::size_of, path::PathBuf};

use amigo_assets::AssetKey;
use amigo_core::AmigoResult;
use amigo_render_api::render_contribution_roles as roles;
use amigo_math::Vec2;
use wgpu::util::DeviceExt;

use crate::renderer::*;

const MAX_PLATE_RELIGHT_LIGHTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlateRelightSkipReason {
    None,
    NoAuxCommand,
    NoSurfaceAsset,
    MissingSurfaceTexture,
    MissingAuxTexture,
    MissingDepthTexture,
    NoLightsFinalOutput,
}

impl PlateRelightSkipReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::NoAuxCommand => "no_aux_command",
            Self::NoSurfaceAsset => "no_surface_asset",
            Self::MissingSurfaceTexture => "missing_surface_texture",
            Self::MissingAuxTexture => "missing_aux_texture",
            Self::MissingDepthTexture => "missing_depth_texture",
            Self::NoLightsFinalOutput => "no_lights_final_output",
        }
    }

    fn fallback_color(self) -> ColorRgba {
        match self {
            Self::None => ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            Self::NoAuxCommand => ColorRgba::new(1.0, 0.0, 1.0, 1.0),
            Self::NoSurfaceAsset => ColorRgba::new(1.0, 0.85, 0.0, 1.0),
            Self::MissingSurfaceTexture => ColorRgba::new(1.0, 0.05, 0.02, 1.0),
            Self::MissingAuxTexture => ColorRgba::new(0.05, 0.25, 1.0, 1.0),
            Self::MissingDepthTexture => ColorRgba::new(0.0, 0.9, 1.0, 1.0),
            Self::NoLightsFinalOutput => ColorRgba::new(0.15, 0.15, 0.15, 1.0),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct PlateRelightUniform {
    canvas: [f32; 4],
    params0: [f32; 4],
    params1: [f32; 4],
    params2: [f32; 4],
    params3: [f32; 4],
    params4: [f32; 4],
    light_pos_rad: [[f32; 4]; MAX_PLATE_RELIGHT_LIGHTS],
    light_color_intensity: [[f32; 4]; MAX_PLATE_RELIGHT_LIGHTS],
    light_dir_type: [[f32; 4]; MAX_PLATE_RELIGHT_LIGHTS],
    light_extra: [[f32; 4]; MAX_PLATE_RELIGHT_LIGHTS],
}

#[derive(Clone, Copy)]
struct PlateRelightLight {
    pos_rad: [f32; 4],
    color_intensity: [f32; 4],
    dir_type: [f32; 4],
    extra: [f32; 4],
}

#[derive(Clone, Copy)]
enum PlateRelightSourcePayload<'a> {
    Beacon(&'a amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand),
    Unsupported,
}

struct WgpuPlateRelightSource<'a> {
    common: amigo_render_api::LightSource2dCommon,
    payload: PlateRelightSourcePayload<'a>,
}

pub(super) fn apply_plate_relight_after_world(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let aux_commands = request.world_2d.depth_maps.aux_commands();
    let aux_count = aux_commands.len();
    let debug_requested = is_plate_relight_debug(request);
    let relight_sources =
        plate_relight_sources_from_frame(request.world_2d.light_sources, request.world_2d.beacons);
    let light_count = relight_sources
        .iter()
        .filter(|source| plate_relight_source_active(source))
        .count();

    let Some(aux) = aux_commands
        .iter()
        .find(|command| command.depth_aux_map.surface_asset.is_some())
    else {
        set_status(
            renderer,
            request,
            PlateRelightSkipReason::NoAuxCommand,
            false,
            debug_requested,
            aux_count,
            light_count,
            None,
            None,
            false,
            false,
            false,
        );
        if debug_requested {
            return draw_plate_relight_debug_fallback(
                renderer,
                target,
                PlateRelightSkipReason::NoAuxCommand.fallback_color(),
            );
        }
        return Ok(());
    };
    let Some(surface_asset) = aux.depth_aux_map.surface_asset.as_ref() else {
        set_status(
            renderer,
            request,
            PlateRelightSkipReason::NoSurfaceAsset,
            false,
            debug_requested,
            aux_count,
            light_count,
            Some(aux),
            None,
            false,
            false,
            false,
        );
        if debug_requested {
            return draw_plate_relight_debug_fallback(
                renderer,
                target,
                PlateRelightSkipReason::NoSurfaceAsset.fallback_color(),
            );
        }
        return Ok(());
    };

    let device = target.device.clone();
    let queue = target.queue.clone();
    let mut scene_source =
        super::offscreen_ops::compatible_offscreen_target(target, "amigo-plate-relight-source");
    renderer.copy_offscreen_to_offscreen(&mut scene_source, &target.view.clone())?;

    let Some((surface_view, sampler)) = texture_view_for_asset(
        renderer,
        &device,
        &queue,
        request,
        &aux.depth_aux_map.asset,
        surface_asset,
        "surface",
    ) else {
        set_status(
            renderer,
            request,
            PlateRelightSkipReason::MissingSurfaceTexture,
            false,
            debug_requested,
            aux_count,
            light_count,
            Some(aux),
            Some(surface_asset),
            false,
            false,
            false,
        );
        if debug_requested {
            return draw_plate_relight_debug_fallback(
                renderer,
                target,
                PlateRelightSkipReason::MissingSurfaceTexture.fallback_color(),
            );
        }
        return Ok(());
    };
    let Some((aux_view, _)) = texture_view_for_asset(
        renderer,
        &device,
        &queue,
        request,
        &aux.depth_aux_map.asset,
        &aux.depth_aux_map.asset,
        "depth_aux",
    ) else {
        set_status(
            renderer,
            request,
            PlateRelightSkipReason::MissingAuxTexture,
            false,
            debug_requested,
            aux_count,
            light_count,
            Some(aux),
            Some(surface_asset),
            true,
            false,
            false,
        );
        if debug_requested {
            return draw_plate_relight_debug_fallback(
                renderer,
                target,
                PlateRelightSkipReason::MissingAuxTexture.fallback_color(),
            );
        }
        return Ok(());
    };
    let depth_commands = request.world_2d.depth_maps.commands();
    let depth_command = depth_commands
        .iter()
        .find(|command| same_canvas_size(command.depth_map.size, aux.depth_aux_map.size))
        .or_else(|| depth_commands.first());

    let mut depth_mode = 0.0f32;
    let mut depth_loaded = false;
    let depth_view = if let Some(command) = depth_command {
        match texture_view_for_asset(
            renderer,
            &device,
            &queue,
            request,
            &aux.depth_aux_map.asset,
            &command.depth_map.asset,
            "depth",
        ) {
            Some((view, _)) => {
                depth_loaded = true;
                depth_mode = if command.depth_map.white_is_near {
                    -1.0
                } else {
                    1.0
                };
                view
            }
            None => {
                if debug_requested {
                    set_status(
                        renderer,
                        request,
                        PlateRelightSkipReason::MissingDepthTexture,
                        false,
                        true,
                        aux_count,
                        light_count,
                        Some(aux),
                        Some(surface_asset),
                        true,
                        true,
                        false,
                    );
                    return draw_plate_relight_debug_fallback(
                        renderer,
                        target,
                        PlateRelightSkipReason::MissingDepthTexture.fallback_color(),
                    );
                }
                aux_view.clone()
            }
        }
    } else {
        aux_view.clone()
    };

    let mut uniforms = plate_relight_uniforms(
        relight_sources.as_slice(),
        aux.depth_aux_map.size,
        depth_mode,
    );
    uniforms.params4[1] = plate_relight_debug_mode(request.camera_debug_view);
    if uniforms.canvas[3] <= 0.0 && uniforms.params4[1] <= 0.5 {
        set_status(
            renderer,
            request,
            PlateRelightSkipReason::NoLightsFinalOutput,
            false,
            false,
            aux_count,
            light_count,
            Some(aux),
            Some(surface_asset),
            true,
            true,
            depth_loaded,
        );
        return Ok(());
    }

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-plate-relight-texture-bind-group"),
        layout: &renderer.wet_reflections_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&scene_source.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&surface_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&aux_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-plate-relight-uniform-buffer"),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-plate-relight-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-plate-relight-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-plate-relight-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-plate-relight-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&renderer.plate_relight_pipeline);
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
    queue.submit(std::iter::once(encoder.finish()));
    set_status(
        renderer,
        request,
        PlateRelightSkipReason::None,
        true,
        false,
        aux_count,
        light_count,
        Some(aux),
        Some(surface_asset),
        true,
        true,
        depth_loaded,
    );

    Ok(())
}

fn plate_relight_uniforms(
    sources: &[WgpuPlateRelightSource<'_>],
    canvas_size: Vec2,
    depth_mode: f32,
) -> PlateRelightUniform {
    let mut uniform = PlateRelightUniform {
        canvas: [
            canvas_size.x.max(1.0),
            canvas_size.y.max(1.0),
            depth_mode,
            0.0,
        ],
        params0: [0.06, 0.16, 1.05, 0.44],
        params1: [1.00, 1.90, 0.08, 0.16],
        params2: [0.30, 3.60, 0.42, 0.62],
        params3: [1.80, 1.05, 0.016, 0.28],
        params4: [28.0, 0.0, 0.0, 0.0],
        light_pos_rad: [[0.0; 4]; MAX_PLATE_RELIGHT_LIGHTS],
        light_color_intensity: [[0.0; 4]; MAX_PLATE_RELIGHT_LIGHTS],
        light_dir_type: [[0.0; 4]; MAX_PLATE_RELIGHT_LIGHTS],
        light_extra: [[0.0; 4]; MAX_PLATE_RELIGHT_LIGHTS],
    };

    let mut count = 0usize;
    for source in sources.iter().filter(|source| plate_relight_source_active(source)) {
        if count >= MAX_PLATE_RELIGHT_LIGHTS {
            break;
        }
        let PlateRelightSourcePayload::Beacon(beacon) = source.payload else {
            continue;
        };
        let light = normalize_beacon_for_plate_relight(beacon, canvas_size);
        uniform.light_pos_rad[count] = light.pos_rad;
        uniform.light_color_intensity[count] = light.color_intensity;
        uniform.light_dir_type[count] = light.dir_type;
        uniform.light_extra[count] = light.extra;
        count += 1;
    }

    uniform.canvas[3] = count as f32;
    uniform
}

fn plate_relight_sources_from_frame<'a>(
    light_sources: &'a [amigo_render_api::LightSource2dCommon],
    beacons: &'a [amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand],
) -> Vec<WgpuPlateRelightSource<'a>> {
    light_sources
        .iter()
        .map(|source| {
            let payload = beacon_payload_for_light_source(source, beacons)
                .map(PlateRelightSourcePayload::Beacon)
                .unwrap_or(PlateRelightSourcePayload::Unsupported);
            let mut common = source.clone();

            if !light_source_has_contribution(
                source,
                amigo_render_api::LightContributionKind2d::RelightPlate,
            ) {
                common.status = amigo_render_api::LightSourceStatus2d::Skipped;
                common.reason = "relight_plate_disabled".to_owned();
                return WgpuPlateRelightSource { common, payload };
            }

            if source.status == amigo_render_api::LightSourceStatus2d::Skipped {
                return WgpuPlateRelightSource { common, payload };
            }

            match payload {
                PlateRelightSourcePayload::Beacon(beacon) => {
                    if beacon.intensity <= 0.001 || beacon.color.a <= 0.001 {
                        common.status = amigo_render_api::LightSourceStatus2d::Skipped;
                        common.reason = "no_visible_light_energy".to_owned();
                    } else {
                        common.status = amigo_render_api::LightSourceStatus2d::Active;
                        common.reason = "relight_plate_active".to_owned();
                    }
                }
                PlateRelightSourcePayload::Unsupported => {
                    common.status = amigo_render_api::LightSourceStatus2d::Skipped;
                    common.reason = match source.emitter_kind {
                        amigo_render_api::LightEmitterKind2d::Beacon => "missing_beacon_payload",
                        amigo_render_api::LightEmitterKind2d::GlobalLight => {
                            "non_spatial_for_plate_relight_v1"
                        }
                        _ => "unsupported_by_plate_relight_v1",
                    }
                    .to_owned();
                }
            }

            WgpuPlateRelightSource { common, payload }
        })
        .collect()
}

fn beacon_payload_for_light_source<'a>(
    source: &amigo_render_api::LightSource2dCommon,
    beacons: &'a [amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand],
) -> Option<&'a amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand> {
    if source.emitter_kind != amigo_render_api::LightEmitterKind2d::Beacon {
        return None;
    }
    beacons
        .iter()
        .find(|beacon| beacon.entity_name == source.owner && source.component_kind == "BeaconLight2D")
}

fn light_source_has_contribution(
    source: &amigo_render_api::LightSource2dCommon,
    contribution: amigo_render_api::LightContributionKind2d,
) -> bool {
    source.contributions.contains(&contribution)
}

fn plate_relight_source_active(source: &WgpuPlateRelightSource<'_>) -> bool {
    source.common.status == amigo_render_api::LightSourceStatus2d::Active
        && matches!(source.payload, PlateRelightSourcePayload::Beacon(_))
        && light_source_has_contribution(
            &source.common,
            amigo_render_api::LightContributionKind2d::RelightPlate,
        )
}

fn normalize_beacon_for_plate_relight(
    beacon: &amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand,
    aux_size: Vec2,
) -> PlateRelightLight {
    let canvas = beacon.viewport_canvas_size.unwrap_or(aux_size);
    let canvas_w = canvas.x.max(1.0);
    let canvas_h = canvas.y.max(1.0);
    let center_uv = Vec2::new(
        (0.5 + beacon.center.x / canvas_w).clamp(0.0, 1.0),
        (0.5 - beacon.center.y / canvas_h).clamp(0.0, 1.0),
    );
    let radius_uv = (beacon.halo_radius_px.max(beacon.core_radius_px * 2.0)
        / canvas_w.max(canvas_h))
    .clamp(0.02, 1.2);
    let distance_m = beacon.distance_m.unwrap_or(1.2).max(0.05);
    let z = beacon
        .z_depth
        .unwrap_or_else(|| relight_light_z_from_distance_m(distance_m))
        .clamp(0.0, 1.0);
    let strength =
        beacon.intensity * beacon.glow_strength.max(0.45) * (1.0 + beacon.pulse.max(0.0) * 0.35);
    let is_spot = beacon.beam_enabled && beacon.beam_strength > 0.001;
    let kind = if is_spot { 1.0 } else { 0.0 };
    let dir = Vec2::new(beacon.rotation_radians.cos(), beacon.rotation_radians.sin());
    let cone = if is_spot {
        beacon
            .beam_width_degrees
            .to_radians()
            .cos()
            .clamp(-1.0, 1.0)
    } else {
        -1.0
    };
    let casts_shadow = if beacon.intensity > 0.001 { 1.0 } else { 0.0 };
    let spec = beacon.camera_response.glare.max(beacon.camera_response.intensity).max(0.7);
    PlateRelightLight {
        pos_rad: [center_uv.x, center_uv.y, z, radius_uv],
        color_intensity: [beacon.color.r, beacon.color.g, beacon.color.b, strength],
        dir_type: [dir.x, dir.y, cone, kind],
        extra: [
            1.0,
            if beacon.intensity > 0.001 && beacon.color.a > 0.001 {
                1.0
            } else {
                0.0
            },
            casts_shadow,
            spec,
        ],
    }
}

fn plate_relight_debug_mode(view: amigo_render_api::CameraDebugView2d) -> f32 {
    match view {
        amigo_render_api::CameraDebugView2d::PlateRelightAuxDepth => 1.0,
        amigo_render_api::CameraDebugView2d::PlateRelightAuxHeight => 2.0,
        amigo_render_api::CameraDebugView2d::PlateRelightAuxOccluder => 3.0,
        amigo_render_api::CameraDebugView2d::PlateRelightAuxValid => 4.0,
        amigo_render_api::CameraDebugView2d::PlateRelightSurfaceReflect => 5.0,
        amigo_render_api::CameraDebugView2d::PlateRelightSurfaceRough => 6.0,
        amigo_render_api::CameraDebugView2d::PlateRelightSurfaceGlass => 7.0,
        amigo_render_api::CameraDebugView2d::PlateRelightSurfaceMask => 8.0,
        amigo_render_api::CameraDebugView2d::PlateRelightEffectiveDepth => 9.0,
        amigo_render_api::CameraDebugView2d::PlateRelightNormal => 10.0,
        amigo_render_api::CameraDebugView2d::PlateRelightOcclusion => 11.0,
        amigo_render_api::CameraDebugView2d::PlateRelightContribution => 12.0,
        amigo_render_api::CameraDebugView2d::PlateRelightShadow => 13.0,
        amigo_render_api::CameraDebugView2d::PlateRelightLightMask => 14.0,
        amigo_render_api::CameraDebugView2d::PlateRelightNdl => 15.0,
        amigo_render_api::CameraDebugView2d::PlateRelightSpecular => 16.0,
        amigo_render_api::CameraDebugView2d::PlateRelightMaterialGate => 17.0,
        amigo_render_api::CameraDebugView2d::PlateRelightLitRaw => 18.0,
        _ => 0.0,
    }
}

fn relight_light_z_from_distance_m(distance_m: f32) -> f32 {
    let distance = if distance_m.is_finite() {
        distance_m.max(0.05)
    } else {
        1.2
    };
    (0.18 + 0.62 / (1.0 + distance * 0.85)).clamp(0.18, 0.80)
}

fn first_light_summary(sources: &[WgpuPlateRelightSource<'_>]) -> String {
    let Some(source) = sources
        .iter()
        .find(|source| plate_relight_source_active(source))
    else {
        return "first_light=-".to_owned();
    };
    let PlateRelightSourcePayload::Beacon(beacon) = source.payload else {
        return "first_light=-".to_owned();
    };
    let canvas = beacon
        .viewport_canvas_size
        .unwrap_or(Vec2::new(1672.0, 941.0));
    let light = normalize_beacon_for_plate_relight(beacon, canvas);
    let kind = if light.dir_type[3] > 0.5 {
        "spot"
    } else {
        "point"
    };
    format!(
        "first_light={} kind={} uv=({:.3},{:.3}) z={:.3} radius={:.3} intensity={:.3} enabled={:.0} casts_shadow={:.0} spec={:.3}",
        beacon.entity_name,
        kind,
        light.pos_rad[0],
        light.pos_rad[1],
        light.pos_rad[2],
        light.pos_rad[3],
        light.color_intensity[3],
        light.extra[1],
        light.extra[2],
        light.extra[3],
    )
}

fn light_list_summary(sources: &[WgpuPlateRelightSource<'_>]) -> String {
    let lights = sources
        .iter()
        .filter(|source| plate_relight_source_active(source))
        .take(4)
        .map(|source| {
            let PlateRelightSourcePayload::Beacon(beacon) = source.payload else {
                return "-".to_owned();
            };
            let canvas = beacon
                .viewport_canvas_size
                .unwrap_or(Vec2::new(1672.0, 941.0));
            let light = normalize_beacon_for_plate_relight(beacon, canvas);
            let kind = if light.dir_type[3] > 0.5 {
                "spot"
            } else {
                "point"
            };
            format!(
                "{}:{} uv=({:.3},{:.3}) z={:.3} radius={:.3} intensity={:.3} shadow={:.0} spec={:.2}",
                beacon.entity_name,
                kind,
                light.pos_rad[0],
                light.pos_rad[1],
                light.pos_rad[2],
                light.pos_rad[3],
                light.color_intensity[3],
                light.extra[2],
                light.extra[3],
            )
        })
        .collect::<Vec<_>>();
    if lights.is_empty() {
        "light_list=-".to_owned()
    } else {
        format!("light_list={}", lights.join(";"))
    }
}

fn is_plate_relight_debug(request: &WgpuFrameRenderRequest<'_>) -> bool {
    request.camera_debug_view.wants_plate_relight_debug()
}

fn set_status(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    reason: PlateRelightSkipReason,
    drawn: bool,
    fallback_drawn: bool,
    aux_count: usize,
    light_count: usize,
    selected_aux: Option<&amigo_focus_depth_plugin::DepthAuxMap2dDrawCommand>,
    surface_asset: Option<&AssetKey>,
    surface_loaded: bool,
    aux_loaded: bool,
    depth_loaded: bool,
) {
    let relight_sources =
        plate_relight_sources_from_frame(request.world_2d.light_sources, request.world_2d.beacons);
    let target_overwrite_hint =
        if request.camera_debug_view.wants_plate_relight_debug() && (drawn || fallback_drawn) {
            "if_image_plain_check_post_world_overwrite"
        } else {
            "-"
        };
    renderer.set_plate_relight_last_summary(format!(
        "plate_relight status={} reason={} debug_view={} debug_mode={} drawn={} fallback_drawn={} aux_commands={} lights={} selected_aux={} aux_asset={} surface_asset={} surface_loaded={} aux_loaded={} depth_loaded={} target_overwrite_hint={} {} {}",
        if drawn { "drawn" } else { "skipped" },
        reason.as_str(),
        request.camera_debug_view.as_str(),
        plate_relight_debug_mode(request.camera_debug_view),
        drawn,
        fallback_drawn,
        aux_count,
        light_count,
        selected_aux.map(|c| c.entity_name.as_str()).unwrap_or("-"),
        selected_aux.map(|c| c.depth_aux_map.asset.as_str()).unwrap_or("-"),
        surface_asset.map(|a| a.as_str()).unwrap_or("-"),
        surface_loaded,
        aux_loaded,
        depth_loaded,
        target_overwrite_hint,
        first_light_summary(relight_sources.as_slice()),
        light_list_summary(relight_sources.as_slice()),
    ));
}

fn same_canvas_size(a: Vec2, b: Vec2) -> bool {
    (a.x - b.x).abs() <= 1.0 && (a.y - b.y).abs() <= 1.0
}

fn draw_plate_relight_debug_fallback(
    renderer: &WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
    color: ColorRgba,
) -> AmigoResult<()> {
    let vertices = vec![
        ColorVertex::new(Vec2::new(-1.0, -1.0), color),
        ColorVertex::new(Vec2::new(1.0, -1.0), color),
        ColorVertex::new(Vec2::new(1.0, 1.0), color),
        ColorVertex::new(Vec2::new(-1.0, -1.0), color),
        ColorVertex::new(Vec2::new(1.0, 1.0), color),
        ColorVertex::new(Vec2::new(-1.0, 1.0), color),
    ];
    let batch = ColorBatch {
        blend_mode: ParticleBlendMode2d::Alpha,
        vertices,
    };
    renderer.render_offscreen_batches(
        target,
        wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        &[],
        &[batch],
        &[],
    )
}

fn texture_view_for_asset(
    renderer: &mut WgpuSceneRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    request: &WgpuFrameRenderRequest<'_>,
    aux_asset: &AssetKey,
    asset: &AssetKey,
    label: &str,
) -> Option<(wgpu::TextureView, wgpu::Sampler)> {
    let image_path = request
        .assets
        .prepared_asset(asset)
        .and_then(|prepared| crate::renderer::assets::resolve_image_path(&prepared))
        .or_else(|| direct_asset_path(aux_asset, asset));
    let image_path = image_path?;
    let texture = renderer.ensure_data_texture_from_path(
        device,
        queue,
        format!("plate-relight:{label}:{}", asset.as_str()),
        image_path,
        true,
        false,
    )?;
    Some((texture._view.clone(), texture._sampler.clone()))
}

fn direct_asset_candidates(aux_asset: &AssetKey, asset: &AssetKey) -> Vec<PathBuf> {
    let key = asset.as_str();
    let mod_id = asset
        .as_str()
        .split('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .or_else(|| {
            aux_asset
                .as_str()
                .split('/')
                .next()
                .filter(|segment| !segment.is_empty())
        })
        .unwrap_or_default();
    let stripped_key = key
        .strip_prefix(&format!("{mod_id}/"))
        .unwrap_or(key)
        .to_owned();

    let mut candidates = vec![
        PathBuf::from(key),
        PathBuf::from("mods").join(key),
        PathBuf::from("..").join(key),
        PathBuf::from("..").join("..").join("mods").join(key),
        PathBuf::from("mods").join("office_desk_mod").join(key),
    ];
    if !mod_id.is_empty() {
        candidates.push(PathBuf::from(&stripped_key));
        candidates.push(PathBuf::from("..").join(mod_id).join(&stripped_key));
        candidates.push(PathBuf::from("mods").join(mod_id).join(&stripped_key));
    }
    candidates
}

fn direct_asset_path(aux_asset: &AssetKey, asset: &AssetKey) -> Option<PathBuf> {
    direct_asset_candidates(aux_asset, asset)
        .into_iter()
        .find(|path| path.is_file())
}

fn fullscreen_vertices() -> [TextureVertex; 6] {
    [
        TextureVertex::new(Vec2::new(-1.0, -1.0), Vec2::new(0.0, 1.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(1.0, -1.0), Vec2::new(1.0, 1.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(-1.0, -1.0), Vec2::new(0.0, 1.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(-1.0, 1.0), Vec2::new(0.0, 0.0), ColorRgba::WHITE),
    ]
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((value as *const T).cast::<u8>(), size_of::<T>()) }
}

fn bytes_of_slice<T>(values: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(values.as_ptr().cast::<u8>(), std::mem::size_of_val(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plate_relight_debug_mode_maps_camera_debug_views() {
        assert_eq!(
            plate_relight_debug_mode(amigo_render_api::CameraDebugView2d::FinalOutput),
            0.0
        );
        assert_eq!(
            plate_relight_debug_mode(amigo_render_api::CameraDebugView2d::PlateRelightAuxDepth),
            1.0
        );
        assert_eq!(
            plate_relight_debug_mode(amigo_render_api::CameraDebugView2d::PlateRelightSurfaceMask),
            8.0
        );
        assert_eq!(
            plate_relight_debug_mode(
                amigo_render_api::CameraDebugView2d::PlateRelightEffectiveDepth
            ),
            9.0
        );
        assert_eq!(
            plate_relight_debug_mode(amigo_render_api::CameraDebugView2d::PlateRelightNormal),
            10.0
        );
        assert_eq!(
            plate_relight_debug_mode(amigo_render_api::CameraDebugView2d::PlateRelightContribution),
            12.0
        );
        assert_eq!(
            plate_relight_debug_mode(amigo_render_api::CameraDebugView2d::PlateRelightShadow),
            13.0
        );
        assert_eq!(
            plate_relight_debug_mode(amigo_render_api::CameraDebugView2d::PlateRelightLightMask),
            14.0
        );
        assert_eq!(
            plate_relight_debug_mode(amigo_render_api::CameraDebugView2d::PlateRelightLitRaw),
            18.0
        );
    }

    #[test]
    fn plate_relight_light_z_from_distance_is_visible_for_cursor_lamp() {
        let near = relight_light_z_from_distance_m(0.2);
        let desk = relight_light_z_from_distance_m(1.2);
        let far = relight_light_z_from_distance_m(150.0);
        assert!(near > desk);
        assert!(desk > far);
        assert!((0.35..=0.62).contains(&desk));
        assert!((0.18..=0.30).contains(&far));
    }

    #[test]
    fn plate_relight_light_uniforms_normalize_canvas_coordinates() {
        let beacon = test_beacon("cursor", 0.0, 0.0);
        let light = normalize_beacon_for_plate_relight(&beacon, Vec2::new(1672.0, 941.0));
        assert!((light.pos_rad[0] - 0.5).abs() < 0.001);
        assert!((light.pos_rad[1] - 0.5).abs() < 0.001);
        assert!(light.pos_rad[3] > 0.02 && light.pos_rad[3] < 1.2);
    }

    #[test]
    fn plate_relight_light_uniforms_use_centered_canvas_coordinates() {
        let beacon = test_beacon("cursor", 420.0, -35.0);
        let light = normalize_beacon_for_plate_relight(&beacon, Vec2::new(1672.0, 941.0));
        assert!((light.pos_rad[0] - 0.751).abs() < 0.002);
        assert!((light.pos_rad[1] - 0.537).abs() < 0.002);
    }

    #[test]
    fn plate_relight_light_uniforms_include_beam_as_spot() {
        let mut beacon = test_beacon("spot", 1244.0, 436.0);
        beacon.beam_enabled = true;
        beacon.beam_strength = 1.0;
        beacon.beam_width_degrees = 35.0;
        let light = normalize_beacon_for_plate_relight(&beacon, Vec2::new(1672.0, 941.0));
        assert_eq!(light.dir_type[3], 1.0);
        assert!(light.dir_type[2] > 0.0);
    }

    #[test]
    fn plate_relight_light_uniforms_keep_cursor_probe_enabled() {
        let beacon = test_beacon("cursor", 420.0, -35.0);
        let light = normalize_beacon_for_plate_relight(&beacon, Vec2::new(1672.0, 941.0));
        assert_eq!(light.extra[1], 1.0);
        assert_eq!(light.extra[2], 1.0);
        assert!(light.color_intensity[3] > 0.0);
    }

    #[test]
    fn plate_relight_light_uniforms_prefer_explicit_z_depth() {
        let mut beacon = test_beacon("cursor", 420.0, -35.0);
        beacon.distance_m = Some(150.0);
        beacon.z_depth = Some(0.66);
        let light = normalize_beacon_for_plate_relight(&beacon, Vec2::new(1672.0, 941.0));
        assert!((light.pos_rad[2] - 0.66).abs() < 0.001);
    }

    #[test]
    fn plate_relight_status_lists_first_lights() {
        let beacon = test_beacon("cursor", 0.0, 0.0);
        let beacons = [beacon];
        let light_sources = [test_light_source_for_beacon(&beacons[0])];
        let sources = plate_relight_sources_from_frame(&light_sources, &beacons);
        let summary = light_list_summary(&sources);
        assert!(summary.contains("light_list=cursor:point"));
        assert!(summary.contains("uv=(0.500,0.500)"));
        assert!(summary.contains("radius="));
    }

    #[test]
    fn plate_relight_sources_use_light_source_contract_for_gating() {
        let mut beacon = test_beacon("disabled", 0.0, 0.0);
        beacon
            .render_contributions
            .set(amigo_render_api::render_contribution_roles::RELIGHT_PLATE, false);

        let beacons = [beacon];
        let light_sources = [test_light_source_for_beacon(&beacons[0])];
        let sources = plate_relight_sources_from_frame(&light_sources, &beacons);

        assert_eq!(sources.len(), 1);
        assert_eq!(
            sources[0].common.status,
            amigo_render_api::LightSourceStatus2d::Skipped
        );
        assert_eq!(sources[0].common.reason, "relight_plate_disabled");
    }

    #[test]
    fn plate_relight_sources_report_zero_energy_as_skipped() {
        let mut beacon = test_beacon("dark", 0.0, 0.0);
        beacon.intensity = 0.0;
        let beacons = [beacon];
        let light_sources = [test_light_source_for_beacon(&beacons[0])];
        let sources = plate_relight_sources_from_frame(&light_sources, &beacons);

        assert_eq!(
            sources[0].common.status,
            amigo_render_api::LightSourceStatus2d::Skipped
        );
        assert_eq!(sources[0].common.reason, "no_visible_light_energy");
    }

    #[test]
    fn plate_relight_sources_report_unsupported_light_kinds() {
        let source = amigo_render_api::LightSource2dCommon::active(
            "ambient".to_owned(),
            "GlobalLight2D",
            amigo_render_api::LightEmitterKind2d::GlobalLight,
            Some("ambient".to_owned()),
            None,
            Some([1.0, 1.0, 1.0, 1.0]),
            Some(1.0),
            Some(1.0),
            Some(1.0),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![amigo_render_api::LightContributionKind2d::RelightPlate],
            "global_light_command",
            None,
        );
        let light_sources = [source];
        let sources = plate_relight_sources_from_frame(&light_sources, &[]);

        assert_eq!(
            sources[0].common.status,
            amigo_render_api::LightSourceStatus2d::Skipped
        );
        assert_eq!(sources[0].common.reason, "non_spatial_for_plate_relight_v1");
    }

    #[test]
    fn direct_asset_candidates_include_mod_root_and_workspace_paths() {
        let aux_asset = AssetKey::new("rotten-club/visual-maps/office-desk/depth_aux_rgba.png");
        let asset = AssetKey::new("rotten-club/visual-maps/office-desk/surface_mask.png");
        let candidates = direct_asset_candidates(&aux_asset, &asset);
        let rendered = candidates
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>();
        assert!(
            rendered
                .iter()
                .any(|path| path.ends_with("visual-maps/office-desk/surface_mask.png"))
        );
        assert!(rendered.iter().any(|path| {
            path.ends_with("mods/rotten-club/visual-maps/office-desk/surface_mask.png")
        }));
        assert!(
            rendered
                .iter()
                .any(|path| path
                    .ends_with("../rotten-club/visual-maps/office-desk/surface_mask.png"))
        );
        assert!(rendered.iter().any(|path| {
            path.ends_with("../../mods/rotten-club/visual-maps/office-desk/surface_mask.png")
        }));
    }

    #[test]
    fn plate_relight_skip_reason_names_are_stable() {
        assert_eq!(
            PlateRelightSkipReason::NoAuxCommand.as_str(),
            "no_aux_command"
        );
        assert_eq!(
            PlateRelightSkipReason::MissingSurfaceTexture.as_str(),
            "missing_surface_texture"
        );
        assert_eq!(
            PlateRelightSkipReason::MissingAuxTexture.as_str(),
            "missing_aux_texture"
        );
    }

    fn test_beacon(
        name: &str,
        x: f32,
        y: f32,
    ) -> amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand {
        amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand {
            entity_name: name.to_owned(),
            render_layer: "lighting.beacons".to_owned(),
            z_index: 0.0,
            center: Vec2::new(x, y),
            color: ColorRgba::new(1.0, 0.8, 0.5, 1.0),
            intensity: 2.0,
            pulse: 0.0,
            core_radius_px: 18.0,
            halo_radius_px: 520.0,
            glow_strength: 0.85,
            rotation_radians: 0.0,
            beam_enabled: false,
            beam_length_px: 0.0,
            beam_width_degrees: 1.0,
            beam_strength: 0.0,
            aberration_px: 0.0,
            
            
            bloom: 0.0,
            camera_response: amigo_camera_optics_plugin::api::CameraOpticalResponse2d { enabled: true, intensity: 1.0, glare: 1.0, ..amigo_camera_optics_plugin::api::CameraOpticalResponse2d::default() },
            distance_m: Some(1.2),
            z_depth: None,
            render_contributions: amigo_render_api::RenderContributionSet::from_pairs([(
                roles::RELIGHT_PLATE,
                true,
            )]),
            viewport_fit: amigo_scene::LayeredImageViewportFit2dSceneCommand::Cover,
            viewport_canvas_size: Some(Vec2::new(1672.0, 941.0)),
        }
    }

    fn test_light_source_for_beacon(
        beacon: &amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand,
    ) -> amigo_render_api::LightSource2dCommon {
        let relight_enabled = beacon.render_contributions.enabled_or(roles::RELIGHT_PLATE, true);
        let mut contributions = Vec::new();
        if relight_enabled {
            contributions.push(amigo_render_api::LightContributionKind2d::RelightPlate);
        }
        if beacon
            .render_contributions
            .enabled_or(roles::BLOOM_SOURCE, true)
        {
            contributions.push(amigo_render_api::LightContributionKind2d::BloomSource);
        }
        if beacon
            .render_contributions
            .enabled_or(roles::CAMERA_FX_SOURCE, true)
        {
            contributions.push(amigo_render_api::LightContributionKind2d::CameraFxSource);
        }
        amigo_render_api::LightSource2dCommon::active(
            beacon.entity_name.clone(),
            "BeaconLight2D",
            amigo_render_api::LightEmitterKind2d::Beacon,
            None,
            Some(beacon.render_layer.clone()),
            Some([beacon.color.r, beacon.color.g, beacon.color.b, beacon.color.a]),
            Some(beacon.intensity),
            Some(beacon.intensity * beacon.color.a),
            Some(1.0),
            None,
            Some(beacon.bloom),
            
            Some(beacon.halo_radius_px.max(beacon.core_radius_px)),
            None,
            beacon.distance_m,
            beacon.z_depth,
            contributions,
            "active_light_emitter",
            Some([beacon.center.x, beacon.center.y]),
        )
    }
}
