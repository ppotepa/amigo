use crate::renderer::{NprDebugOverlay3d, Viewport};

use super::{NprGpuFramePlan3d, NprGpuMeshJob3d};

pub(crate) fn npr_gpu_trace_enabled(frame_index: u64) -> bool {
    if npr_gpu_trace_env_is_false("AMIGO_NPR_GPU_TRACE") {
        return false;
    }
    frame_index <= 4
        || std::env::var("AMIGO_NPR_GPU_TRACE")
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
}

pub(crate) fn npr_gpu_trace_frame_start(
    frame_index: u64,
    frame_jobs: &[NprGpuMeshJob3d],
    viewport: &Viewport,
    overlay: Option<NprDebugOverlay3d>,
) {
    npr_gpu_trace_header(frame_index);
    npr_gpu_trace_line(
        frame_index,
        "START",
        format!(
            "begin jobs={} viewport=({:.0}x{:.0}) overlay={:?}",
            frame_jobs.len(),
            viewport.size().x,
            viewport.size().y,
            overlay
        ),
    );
    for (index, job) in frame_jobs.iter().enumerate() {
        npr_gpu_trace_line(
            frame_index,
            "JOB",
            format!(
                "job[{index}] entity={} mesh={} vertices={} triangles={} edges={} strategy={} preset={} tool={} visibility_max={:.1} fill={:?}",
                job.entity_name,
                job.mesh_key,
                job.geometry.vertex_count(),
                job.geometry.triangle_count(),
                job.geometry.edge_count(),
                job.settings.render_strategy.as_str(),
                job.settings.style_preset.as_str(),
                job.settings.stroke_tool.as_str(),
                job.settings.visibility_max_dimension_px,
                job.settings.fill_mode,
            ),
        );
    }
}

pub(crate) fn npr_gpu_trace_frame_plan(frame_index: u64, frame_plan: NprGpuFramePlan3d) {
    npr_gpu_trace_line(
        frame_index,
        "ALLOC",
        format!(
            "capacities bytes projected={} visible={} endpoint_heads={} endpoint_entries={} path_links={} path_segments={} path_states={} aggregated_paths={} stroke_segments={} uniform_size={}",
            frame_plan.allocated_projected_capacity(),
            frame_plan.allocated_visible_segments_capacity(),
            frame_plan.allocated_endpoint_heads_capacity(),
            frame_plan.allocated_endpoint_entries_capacity(),
            frame_plan.allocated_path_links_capacity(),
            frame_plan.allocated_path_segments_capacity(),
            frame_plan.allocated_path_states_capacity(),
            frame_plan.allocated_aggregated_paths_capacity(),
            frame_plan.allocated_stroke_segments_capacity(),
            frame_plan.uniform_size,
        ),
    );
}

pub(crate) fn npr_gpu_trace_header(frame_index: u64) {
    if frame_index != 1 {
        return;
    }
    if npr_gpu_trace_clear_enabled() {
        print!("\x1b[2J\x1b[H");
    }
    npr_gpu_trace_line(frame_index, "START", "GPU NPR realtime trace");
    npr_gpu_trace_line(
        frame_index,
        "INFO",
        "env: AMIGO_NPR_GPU_TRACE=1 keeps logging, AMIGO_NPR_GPU_TRACE_CLEAR=0 disables clear, AMIGO_NPR_GPU_TRACE_COLOR=0 disables colors",
    );
}

pub(crate) fn npr_gpu_trace_line(frame_index: u64, level: &str, message: impl AsRef<str>) {
    let message = message.as_ref();
    if npr_gpu_trace_color_enabled() {
        let color = npr_gpu_trace_level_color(level);
        println!(
            "{color}[npr-gpu]\x1b[0m \x1b[2mframe={frame_index:04}\x1b[0m {color}{level:<5}\x1b[0m {message}"
        );
    } else {
        println!("[npr-gpu] frame={frame_index:04} {level:<5} {message}");
    }
}

fn npr_gpu_trace_clear_enabled() -> bool {
    !npr_gpu_trace_env_is_false("AMIGO_NPR_GPU_TRACE_CLEAR")
}

fn npr_gpu_trace_color_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none()
        && !npr_gpu_trace_env_is_false("AMIGO_NPR_GPU_TRACE_COLOR")
}

fn npr_gpu_trace_env_is_false(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
        })
        .unwrap_or(false)
}

fn npr_gpu_trace_level_color(level: &str) -> &'static str {
    match level {
        "START" => "\x1b[1;36m",
        "INFO" => "\x1b[36m",
        "JOB" => "\x1b[32m",
        "STEP" => "\x1b[33m",
        "ALLOC" => "\x1b[35m",
        "WRITE" => "\x1b[34m",
        "OK" => "\x1b[32m",
        _ => "\x1b[37m",
    }
}
