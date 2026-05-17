import "mod:fx/ids" as ids;
import "mod:fx/math" as math;

fn reset(world_api) {
    world_api.state.set_float("rain.next_gust", world_api.random.range(8.0, 12.0));
    world_api.state.set_float("rain.gust_start", -100.0);
    world_api.state.set_float("rain.gust_until", -100.0);
    world_api.state.set_float("rain.gust_duration", 1.0);
    world_api.state.set_float("rain.gust_strength", 0.0);
    world_api.state.set_float("rain.gust_dir", 1.0);

    world_api.particles.set_intensity(ids::rain_horizon_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_far_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_mid_far_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_mid_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_street_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_near_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_close_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_front_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_lens_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_micro_entity(), 0.0);
    world_api.particles.set_wind(ids::rain_horizon_entity(), 58.0, -840.0, 0.34);
    world_api.particles.set_wind(ids::rain_far_entity(), 78.0, -980.0, 0.44);
    world_api.particles.set_wind(ids::rain_mid_far_entity(), 104.0, -1260.0, 0.52);
    world_api.particles.set_wind(ids::rain_mid_entity(), 126.0, -1560.0, 0.60);
    world_api.particles.set_wind(ids::rain_street_entity(), 164.0, -1880.0, 0.66);
    world_api.particles.set_wind(ids::rain_near_entity(), 190.0, -2160.0, 0.72);
    world_api.particles.set_wind(ids::rain_close_entity(), 216.0, -2460.0, 0.78);
    world_api.particles.set_wind(ids::rain_front_entity(), 240.0, -2820.0, 0.84);
    world_api.particles.set_wind(ids::rain_lens_entity(), 270.0, -3060.0, 0.90);
    world_api.particles.set_wind(ids::rain_micro_entity(), 310.0, -3360.0, 0.96);
}


fn update(world_api, time, far_light, mid_light, near_light) {
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

    let rain_onset = math::fade(time, 12.0, 2.0);

    world_api.particles.set_intensity(ids::rain_horizon_entity(), math::clamp01((far_light * 0.66 + gust * 0.14) * rain_onset));
    world_api.particles.set_intensity(ids::rain_far_entity(), math::clamp01((far_light * 0.62 + gust * 0.16) * rain_onset));
    world_api.particles.set_intensity(ids::rain_mid_far_entity(), math::clamp01(((far_light * 0.42 + mid_light * 0.24) + gust * 0.15) * rain_onset));
    world_api.particles.set_intensity(ids::rain_mid_entity(), math::clamp01((mid_light * 0.52 + gust * 0.14) * rain_onset));
    world_api.particles.set_intensity(ids::rain_street_entity(), math::clamp01(((mid_light * 0.28 + near_light * 0.28) + gust * 0.13) * rain_onset));
    world_api.particles.set_intensity(ids::rain_near_entity(), math::clamp01((near_light * 0.42 + gust * 0.12) * rain_onset));
    world_api.particles.set_intensity(ids::rain_close_entity(), math::clamp01((near_light * 0.28 + gust * 0.10) * rain_onset));
    world_api.particles.set_intensity(ids::rain_front_entity(), math::clamp01(((far_light + mid_light + near_light) * 0.08 + gust * 0.08) * rain_onset));
    world_api.particles.set_intensity(ids::rain_lens_entity(), math::clamp01((0.18 + (far_light + mid_light + near_light) * 0.11 + gust * 0.12) * rain_onset));
    world_api.particles.set_intensity(ids::rain_micro_entity(), math::clamp01((0.12 + (far_light + mid_light + near_light) * 0.08 + gust * 0.10) * rain_onset));
    world_api.particles.set_wind(
        ids::rain_horizon_entity(),
        58.0 + dir * gust * 640.0,
        -840.0 - gust * 132.0,
        0.34 + gust * 0.58
    );
    world_api.particles.set_wind(
        ids::rain_far_entity(),
        78.0 + dir * gust * 780.0,
        -980.0 - gust * 162.0,
        0.44 + gust * 0.66
    );
    world_api.particles.set_wind(
        ids::rain_mid_far_entity(),
        104.0 + dir * gust * 1020.0,
        -1260.0 - gust * 220.0,
        0.52 + gust * 0.74
    );
    world_api.particles.set_wind(
        ids::rain_mid_entity(),
        126.0 + dir * gust * 1290.0,
        -1560.0 - gust * 276.0,
        0.60 + gust * 0.81
    );
    world_api.particles.set_wind(
        ids::rain_street_entity(),
        164.0 + dir * gust * 1560.0,
        -1880.0 - gust * 340.0,
        0.66 + gust * 0.88
    );
    world_api.particles.set_wind(
        ids::rain_near_entity(),
        190.0 + dir * gust * 1860.0,
        -2160.0 - gust * 390.0,
        0.72 + gust * 0.96
    );
    world_api.particles.set_wind(
        ids::rain_close_entity(),
        216.0 + dir * gust * 1980.0,
        -2460.0 - gust * 405.0,
        0.78 + gust * 0.92
    );
    world_api.particles.set_wind(
        ids::rain_front_entity(),
        240.0 + dir * gust * 2100.0,
        -2820.0 - gust * 420.0,
        0.84 + gust * 0.86
    );
    world_api.particles.set_wind(
        ids::rain_lens_entity(),
        270.0 + dir * gust * 2200.0,
        -3060.0 - gust * 430.0,
        0.90 + gust * 0.82
    );
    world_api.particles.set_wind(
        ids::rain_micro_entity(),
        310.0 + dir * gust * 2300.0,
        -3360.0 - gust * 440.0,
        0.96 + gust * 0.78
    );
}



