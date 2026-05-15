import "mod:fx/ids" as ids;
import "mod:fx/math" as math;

fn reset(world_api) {
    world_api.state.set_float("rain.next_gust", world_api.random.range(8.0, 12.0));
    world_api.state.set_float("rain.gust_start", -100.0);
    world_api.state.set_float("rain.gust_until", -100.0);
    world_api.state.set_float("rain.gust_duration", 1.0);
    world_api.state.set_float("rain.gust_strength", 0.0);
    world_api.state.set_float("rain.gust_dir", 1.0);

    world_api.particles.set_intensity(ids::rain_far_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_mid_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_near_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_front_entity(), 0.0);
    world_api.particles.set_wind(ids::rain_far_entity(), 68.0, -960.0, 0.36);
    world_api.particles.set_wind(ids::rain_mid_entity(), 112.0, -1500.0, 0.48);
    world_api.particles.set_wind(ids::rain_near_entity(), 150.0, -2040.0, 0.54);
    world_api.particles.set_wind(ids::rain_front_entity(), 240.0, -2820.0, 0.66);
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

    world_api.particles.set_intensity(ids::rain_far_entity(), math::clamp01((far_light * 0.62 + gust * 0.16) * rain_onset));
    world_api.particles.set_intensity(ids::rain_mid_entity(), math::clamp01((mid_light * 0.52 + gust * 0.14) * rain_onset));
    world_api.particles.set_intensity(ids::rain_near_entity(), math::clamp01((near_light * 0.42 + gust * 0.12) * rain_onset));
    world_api.particles.set_intensity(ids::rain_front_entity(), math::clamp01(((far_light + mid_light + near_light) * 0.08 + gust * 0.08) * rain_onset));
    world_api.particles.set_wind(
        ids::rain_far_entity(),
        68.0 + dir * gust * 780.0,
        -960.0 - gust * 162.0,
        0.36 + gust * 0.66
    );
    world_api.particles.set_wind(
        ids::rain_mid_entity(),
        112.0 + dir * gust * 1290.0,
        -1500.0 - gust * 276.0,
        0.48 + gust * 0.81
    );
    world_api.particles.set_wind(
        ids::rain_near_entity(),
        150.0 + dir * gust * 1860.0,
        -2040.0 - gust * 390.0,
        0.54 + gust * 0.96
    );
    world_api.particles.set_wind(
        ids::rain_front_entity(),
        210.0 + dir * gust * 2100.0,
        -2520.0 - gust * 420.0,
        0.56 + gust * 0.86
    );
}



