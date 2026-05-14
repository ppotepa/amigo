use crate::renderer::*;

const WET_REFLECTIONS_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct WetReflectionsUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    mask_invert: f32,
    blur_px: f32,
    distortion_px: f32,
    shimmer_strength: f32,
    ripple_strength: f32,
    wet_darken: f32,
    specular_boost: f32,
    edge_power: f32,
    light_reflection_strength: f32,
    foreground_strength: f32,
    background_strength: f32,
    horizon_y: f32,
    noise_scale: f32,
    noise_speed: f32,
    ripple_speed: f32,
    _pad0: vec2<f32>,
}

@group(0) @binding(0) var world_tex: texture_2d<f32>;
@group(0) @binding(1) var mask_tex: texture_2d<f32>;
@group(0) @binding(2) var edge_tex: texture_2d<f32>;
@group(0) @binding(3) var reflection_color_tex: texture_2d<f32>;
@group(0) @binding(4) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: WetReflectionsUniform;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn hash_noise(uv: vec2<f32>, time: f32) -> vec2<f32> {
    let n1 = fract(sin(dot(uv + vec2<f32>(time, 0.0), vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let n2 = fract(sin(dot(uv + vec2<f32>(0.0, time), vec2<f32>(39.3468, 11.135))) * 24634.6345);
    return vec2<f32>(n1, n2);
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let base = textureSample(world_tex, source_sampler, input.uv);
    let mask_sample = textureSample(mask_tex, source_sampler, input.uv);
    let edge_sample = textureSample(edge_tex, source_sampler, input.uv);
    let reflection_color = textureSample(reflection_color_tex, source_sampler, input.uv);

    var coverage = luminance(mask_sample.rgb);
    if (uniforms.mask_invert > 0.5) {
        coverage = 1.0 - coverage;
    }
    coverage = clamp(coverage * mask_sample.a, 0.0, 1.0);

    let edge = pow(clamp(luminance(edge_sample.rgb), 0.0, 1.0), uniforms.edge_power);
    let perspective = mix(uniforms.background_strength, uniforms.foreground_strength, smoothstep(uniforms.horizon_y, 1.0, input.uv.y));
    coverage = clamp(pow(coverage, 0.45) * perspective, 0.0, 1.0);

    let noise = hash_noise(input.uv * uniforms.noise_scale, uniforms.time_seconds * uniforms.noise_speed);
    let ripple = sin((input.uv.y + uniforms.time_seconds * uniforms.ripple_speed) * 40.0) * uniforms.ripple_strength;
    let offset = (noise - vec2<f32>(0.5, 0.5)) * uniforms.distortion_px / uniforms.resolution;
    let reflection_uv = vec2<f32>(
        clamp(input.uv.x + offset.x * coverage, 0.0, 1.0),
        clamp(1.0 - input.uv.y + offset.y * coverage + ripple * coverage, 0.0, 1.0),
    );

    var blurred = textureSample(world_tex, source_sampler, reflection_uv);
    blurred += textureSample(world_tex, source_sampler, reflection_uv + vec2<f32>(1.0, 0.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv - vec2<f32>(1.0, 0.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv + vec2<f32>(0.0, 1.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv - vec2<f32>(0.0, 1.0) / uniforms.resolution * uniforms.blur_px);
    blurred *= 0.2;

    let light_source = mix(blurred.rgb, reflection_color.rgb, reflection_color.a);
    let light_strength = luminance(light_source);
    let specular = coverage * edge * light_strength * uniforms.specular_boost;
    let wet_mix = clamp(coverage * (0.92 + edge * 0.82), 0.0, 1.0);

    var final_rgb = mix(base.rgb, blurred.rgb, wet_mix);
    final_rgb += light_source * coverage * edge * uniforms.light_reflection_strength;
    final_rgb += vec3<f32>(specular * (1.0 + noise.x * uniforms.shimmer_strength * 2.0));
    let wet_brighten = vec3<f32>(0.06, 0.07, 0.08) + light_source * 0.12;
    final_rgb = mix(final_rgb, final_rgb * (1.0 - uniforms.wet_darken) + wet_brighten, coverage);

    return vec4<f32>(final_rgb, 1.0);
}
"#;

const FILM_NOISE_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct FilmNoiseUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    iso: f32,
    grain_size: f32,
    chroma_noise: f32,
    color_shift: f32,
    contrast: f32,
    saturation: f32,
    flicker: f32,
    vignette: f32,
    opacity: f32,
    seed: f32,
    _pad0: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: FilmNoiseUniform;

fn hash12(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let base = textureSample(source_tex, source_sampler, input.uv);
    let luma = luminance(base.rgb);
    let iso_stops = max(log2(max(uniforms.iso, 50.0) / 100.0), 0.0);
    let grain_strength = iso_stops * 0.045;
    let shadow_weight = 1.0 - smoothstep(0.58, 1.0, luma);
    let grain_density = max(20.0, 320.0 / max(uniforms.grain_size, 0.25));
    let cell = floor(input.uv * uniforms.resolution / grain_density);
    let time_phase = floor(uniforms.time_seconds * 24.0);
    let mono = hash12(cell + vec2<f32>(time_phase + uniforms.seed, uniforms.seed * 0.17)) - 0.5;
    let chroma_r = hash12(cell + vec2<f32>(uniforms.seed * 1.7, time_phase)) - 0.5;
    let chroma_b = hash12(cell + vec2<f32>(time_phase, uniforms.seed * 2.3)) - 0.5;
    let flicker = 1.0 + (hash12(vec2<f32>(time_phase, uniforms.seed)) - 0.5) * uniforms.flicker;
    let vignette_distance = distance(input.uv, vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.34, 0.78, vignette_distance) * uniforms.vignette;

    var graded = (base.rgb - vec3<f32>(0.5)) * uniforms.contrast + vec3<f32>(0.5);
    let gray = vec3<f32>(luminance(graded));
    graded = mix(gray, graded, uniforms.saturation);
    graded.r += uniforms.color_shift * 0.035 * shadow_weight;
    graded.b -= uniforms.color_shift * 0.025 * shadow_weight;

    let grain = mono * grain_strength * shadow_weight * flicker;
    let chroma = vec3<f32>(chroma_r, 0.0, chroma_b) * uniforms.chroma_noise * grain_strength;
    let film_rgb = clamp((graded + vec3<f32>(grain) + chroma) * vignette, vec3<f32>(0.0), vec3<f32>(1.0));
    let rgb = mix(base.rgb, film_rgb, uniforms.opacity);
    return vec4<f32>(rgb, base.a);
}
"#;

const DIRTY_BLOOM_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct DirtyBloomUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    threshold: f32,
    strength: f32,
    small_radius_px: f32,
    medium_radius_px: f32,
    large_radius_px: f32,
    dirty_noise: f32,
    halation_strength: f32,
    reflection_smear_x_px: f32,
    reflection_smear_y_px: f32,
    seed: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: DirtyBloomUniform;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn hash12(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn bright_sample(uv: vec2<f32>) -> vec3<f32> {
    let color = textureSample(source_tex, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).rgb;
    let lift = smoothstep(uniforms.threshold, 1.0, luminance(color));
    return color * lift;
}

fn blur_ring(uv: vec2<f32>, radius_px: f32) -> vec3<f32> {
    let texel = vec2<f32>(1.0) / max(uniforms.resolution, vec2<f32>(1.0));
    let r = radius_px * texel;
    var color = bright_sample(uv) * 0.2;
    color += bright_sample(uv + vec2<f32>( r.x,  0.0)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x,  0.0)) * 0.1;
    color += bright_sample(uv + vec2<f32>( 0.0,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( 0.0, -r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( r.x,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( r.x, -r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x, -r.y)) * 0.1;
    return color;
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let base = textureSample(source_tex, source_sampler, input.uv);
    let bloom_small = blur_ring(input.uv, uniforms.small_radius_px) * 0.45;
    let bloom_medium = blur_ring(input.uv, uniforms.medium_radius_px) * 0.65;
    let bloom_large = blur_ring(input.uv, uniforms.large_radius_px) * 0.35;
    let texel = vec2<f32>(1.0) / max(uniforms.resolution, vec2<f32>(1.0));
    let smear = (
        bright_sample(input.uv + vec2<f32>(uniforms.reflection_smear_x_px, uniforms.reflection_smear_y_px) * texel) +
        bright_sample(input.uv + vec2<f32>(-uniforms.reflection_smear_x_px, uniforms.reflection_smear_y_px) * texel) +
        bright_sample(input.uv + vec2<f32>(0.0, uniforms.reflection_smear_y_px * 1.6) * texel)
    ) * 0.18;
    let dirty = 1.0 - uniforms.dirty_noise + hash12(floor(input.uv * uniforms.resolution * 0.18) + vec2<f32>(uniforms.seed, floor(uniforms.time_seconds * 12.0))) * uniforms.dirty_noise;
    let hot = blur_ring(input.uv, 6.0) * smoothstep(0.78, 1.0, luminance(bright_sample(input.uv)));
    let halation = hot * vec3<f32>(1.0, 0.25, 0.35) * uniforms.halation_strength;
    let rgb = clamp(base.rgb + ((bloom_small + bloom_medium + bloom_large + smear) * dirty * uniforms.strength) + halation, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb, base.a);
}
"#;

const CRT_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct CrtUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    scanline_opacity: f32,
    scanline_frequency_px: f32,
    rgb_split_px: f32,
    curvature: f32,
    vignette: f32,
    phosphor_mask: f32,
    brightness_compensation: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: CrtUniform;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let centered = input.uv * 2.0 - vec2<f32>(1.0);
    let curve = dot(centered, centered) * uniforms.curvature;
    let uv = input.uv + centered * curve;
    let inside = step(0.0, uv.x) * step(uv.x, 1.0) * step(0.0, uv.y) * step(uv.y, 1.0);
    let split = vec2<f32>(uniforms.rgb_split_px / max(uniforms.resolution.x, 1.0), 0.0);
    let r = textureSample(source_tex, source_sampler, clamp(uv + split, vec2<f32>(0.0), vec2<f32>(1.0))).r;
    let g = textureSample(source_tex, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).g;
    let b = textureSample(source_tex, source_sampler, clamp(uv - split, vec2<f32>(0.0), vec2<f32>(1.0))).b;
    var rgb = vec3<f32>(r, g, b);
    let scanline = 1.0 - uniforms.scanline_opacity * (0.5 + 0.5 * sin(uv.y * uniforms.resolution.y * 6.2831853 / max(uniforms.scanline_frequency_px, 0.5)));
    let mask = 1.0 - uniforms.phosphor_mask * (0.5 + 0.5 * sin(uv.x * uniforms.resolution.x * 2.0943951));
    let vignette_distance = distance(input.uv, vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.35, 0.82, vignette_distance) * uniforms.vignette;
    rgb = clamp(rgb * scanline * mask * vignette * uniforms.brightness_compensation * inside, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb, 1.0);
}
"#;

impl WgpuSceneRenderer {
    pub fn new(surface: &WgpuSurfaceState) -> Self {
        Self::new_with_device(&surface.device, surface.config.format)
    }

    pub fn new_for_offscreen(target: &WgpuOffscreenTarget) -> Self {
        Self::new_with_device(&target.device, target.format)
    }

    fn new_with_device(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-color-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_SHADER)),
        });
        let color_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-color-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let color_alpha_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[ColorVertex::layout()],
        );
        let color_additive_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-additive-pipeline",
            additive_blend_state(),
            &[ColorVertex::layout()],
        );
        let color_multiply_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-multiply-pipeline",
            multiply_blend_state(),
            &[ColorVertex::layout()],
        );
        let color_screen_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-screen-pipeline",
            screen_blend_state(),
            &[ColorVertex::layout()],
        );

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-scene-texture-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-texture-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TEXTURE_SHADER)),
        });
        let texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-texture-pipeline-layout"),
                bind_group_layouts: &[Some(&texture_bind_group_layout)],
                immediate_size: 0,
            });
        let texture_alpha_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[TextureVertex::layout()],
        );
        let texture_additive_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-additive-pipeline",
            additive_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_multiply_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-multiply-pipeline",
            multiply_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_screen_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-screen-pipeline",
            screen_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_lighten_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-lighten-pipeline",
            lighten_blend_state(),
            &[TextureVertex::layout()],
        );

        let wet_reflections_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-scene-wet-reflections-texture-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let wet_reflections_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-scene-wet-reflections-uniform-bind-group-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let wet_reflections_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-wet-reflections-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WET_REFLECTIONS_SHADER)),
        });
        let wet_reflections_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-wet-reflections-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&wet_reflections_texture_bind_group_layout),
                    Some(&wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let wet_reflections_pipeline = create_color_pipeline(
            device,
            &wet_reflections_shader,
            &wet_reflections_pipeline_layout,
            format,
            "amigo-scene-wet-reflections-pipeline",
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            &[TextureVertex::layout()],
        );
        let film_noise_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-film-noise-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(FILM_NOISE_SHADER)),
        });
        let film_noise_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-film-noise-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&texture_bind_group_layout),
                    Some(&wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let film_noise_pipeline = create_color_pipeline(
            device,
            &film_noise_shader,
            &film_noise_pipeline_layout,
            format,
            "amigo-scene-film-noise-pipeline",
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            &[TextureVertex::layout()],
        );
        let dirty_bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-dirty-bloom-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(DIRTY_BLOOM_SHADER)),
        });
        let dirty_bloom_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-dirty-bloom-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&texture_bind_group_layout),
                    Some(&wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let dirty_bloom_pipeline = create_color_pipeline(
            device,
            &dirty_bloom_shader,
            &dirty_bloom_pipeline_layout,
            format,
            "amigo-scene-dirty-bloom-pipeline",
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            &[TextureVertex::layout()],
        );
        let crt_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-crt-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(CRT_SHADER)),
        });
        let crt_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-scene-crt-pipeline-layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&wet_reflections_uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });
        let crt_pipeline = create_color_pipeline(
            device,
            &crt_shader,
            &crt_pipeline_layout,
            format,
            "amigo-scene-crt-pipeline",
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            &[TextureVertex::layout()],
        );

        Self {
            color_alpha_pipeline,
            color_additive_pipeline,
            color_multiply_pipeline,
            color_screen_pipeline,
            texture_alpha_pipeline,
            texture_additive_pipeline,
            texture_multiply_pipeline,
            texture_screen_pipeline,
            texture_lighten_pipeline,
            texture_bind_group_layout,
            wet_reflections_texture_bind_group_layout,
            wet_reflections_uniform_bind_group_layout,
            wet_reflections_pipeline,
            dirty_bloom_pipeline,
            film_noise_pipeline,
            crt_pipeline,
            texture_cache: BTreeMap::new(),
            lightmap_2d_image_cache: BTreeMap::new(),
            font_atlas_cache: BTreeMap::new(),
            frame_graph_executor: crate::renderer::graph::WgpuFrameGraphExecutor::default(),
            emergency_overlay_lines: Vec::new(),
        }
    }
}
