import "mod:main_menu/ids" as ids;
import "mod:main_menu/math" as math;

fn reset(world_api) {
    world_api.state.set_float("rain.next_gust", world_api.random.range(8.0, 12.0));
    world_api.state.set_float("rain.gust_start", -100.0);
    world_api.state.set_float("rain.gust_until", -100.0);
    world_api.state.set_float("rain.gust_duration", 1.0);
    world_api.state.set_float("rain.gust_strength", 0.0);
    world_api.state.set_float("rain.gust_dir", 1.0);

    world_api.particles.set_intensity(ids::rain_10m_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_5m_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_1m_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_super_near_entity(), 0.0);
    world_api.particles.set_wind(ids::rain_10m_entity(), 126.0, -1560.0, 0.56);
    world_api.particles.set_wind(ids::rain_5m_entity(), 180.0, -2060.0, 0.72);
    world_api.particles.set_wind(ids::rain_1m_entity(), 250.0, -2840.0, 0.84);
    world_api.particles.set_wind(ids::rain_super_near_entity(), 310.0, -3360.0, 0.94);
    set_emitters_active(world_api, world_api.state.get_bool("rain.enabled"));
}

fn set_emitters_active(world_api, active) {
    world_api.particles.set_active(ids::rain_10m_entity(), active);
    world_api.particles.set_active(ids::rain_5m_entity(), active);
    world_api.particles.set_active(ids::rain_1m_entity(), active);
    world_api.particles.set_active(ids::rain_super_near_entity(), active);
}

fn update(world_api, time, far_light, mid_light, near_light) {
    if !world_api.state.get_bool("rain.enabled") {
        world_api.particles.set_intensity(ids::rain_10m_entity(), 0.0);
        world_api.particles.set_intensity(ids::rain_5m_entity(), 0.0);
        world_api.particles.set_intensity(ids::rain_1m_entity(), 0.0);
        world_api.particles.set_intensity(ids::rain_super_near_entity(), 0.0);
        set_emitters_active(world_api, false);
        return;
    }

    set_emitters_active(world_api, true);

    if time >= world_api.state.get_float("rain.next_gust") {
        let duration = world_api.random.range(0.8, 2.4);
        let strength = world_api.random.range(0.35, 1.0);
        let direction = if world_api.random.range(0.0, 1.0) < 0.18 {
            -1.0
        } else {
            1.0
        };

        world_api.state.set_float("rain.gust_start", time);
        world_api.state.set_float("rain.gust_until", time + duration);
        world_api.state.set_float("rain.gust_duration", duration);
        world_api.state.set_float("rain.gust_strength", strength);
        world_api.state.set_float("rain.gust_dir", direction);
        world_api.state.set_float("rain.next_gust", time + duration + world_api.random.range(4.0, 10.0));
    }

    let gust = if time < world_api.state.get_float("rain.gust_until") {
        let phase = math::clamp01((time - world_api.state.get_float("rain.gust_start")) / world_api.state.get_float("rain.gust_duration"));
        let envelope = (phase * 3.14159).sin();
        envelope * world_api.state.get_float("rain.gust_strength")
    } else {
        0.0
    };
    let dir = world_api.state.get_float("rain.gust_dir");
    let rain_onset = math::clamp01(world_api.state.get_float("intro.rain_intensity"));

    world_api.particles.set_intensity(ids::rain_10m_entity(), math::clamp01(((far_light * 0.48 + mid_light * 0.18) + gust * 0.12) * rain_onset));
    world_api.particles.set_intensity(ids::rain_5m_entity(), math::clamp01(((mid_light * 0.36 + near_light * 0.22) + gust * 0.12) * rain_onset));
    world_api.particles.set_intensity(ids::rain_1m_entity(), math::clamp01((0.08 + near_light * 0.34 + gust * 0.10) * rain_onset));
    world_api.particles.set_intensity(ids::rain_super_near_entity(), math::clamp01((0.16 + (far_light + mid_light + near_light) * 0.10 + gust * 0.10) * rain_onset));
    world_api.particles.set_wind(ids::rain_10m_entity(), 126.0 + dir * gust * 920.0, -1560.0 - gust * 210.0, 0.56 + gust * 0.68);
    world_api.particles.set_wind(ids::rain_5m_entity(), 180.0 + dir * gust * 1420.0, -2060.0 - gust * 330.0, 0.72 + gust * 0.86);
    world_api.particles.set_wind(ids::rain_1m_entity(), 250.0 + dir * gust * 2040.0, -2840.0 - gust * 410.0, 0.84 + gust * 0.86);
    world_api.particles.set_wind(ids::rain_super_near_entity(), 310.0 + dir * gust * 2300.0, -3360.0 - gust * 440.0, 0.94 + gust * 0.78);
}
