// Clean source-scene choreography. All values are recomputed from absolute
// scene time; this script intentionally owns no UI and no renderer state.

fn clamp01(v) { if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v } }
fn lerp(a, b, t) { a + (b - a) * t }
fn smoothstep(v) { let t = clamp01(v); t * t * (3.0 - 2.0 * t) }
fn smootherstep(v) { let t = clamp01(v); t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }
fn atan_approx(v) { v / (1.0 + 0.28 * v * v) }
fn atan2_approx(y, x) {
  if x > 0.0 { atan_approx(y / x) }
  else if x < 0.0 && y >= 0.0 { atan_approx(y / x) + 3.141592653589793 }
  else if x < 0.0 { atan_approx(y / x) - 3.141592653589793 }
  else if y > 0.0 { 1.5707963267948966 }
  else if y < 0.0 { -1.5707963267948966 }
  else { 0.0 }
}
fn sin_approx(x) { x - x * x * x / 6.0 + x * x * x * x * x / 120.0 }
fn cos_approx(x) { 1.0 - x * x / 2.0 + x * x * x * x / 24.0 }
fn timeline_time(world) { (world.time.elapsed() - world.state.get_float("npr-city.start")) % 30.0 }

fn pose(world, entity, x, y, z, yaw) {
  world.entities.set_pose_3d(entity, x, y, z, 0.0, yaw, 0.0);
}

fn camera(world, x, y, z, tx, ty, tz, fov) {
  let dx = tx - x;
  let dy = ty - y;
  let dz = tz - z;
  let horizontal = if dx * dx + dz * dz > 0.0001 { dx * dx + dz * dz } else { 0.0001 };
  let pitch = atan_approx(dy / horizontal);
  let yaw = atan2_approx(-dx, -dz);
  world.entities.set_pose_3d("city-camera", x, y, z, pitch, yaw, 0.0);
  world.entities.set_property_float("city-camera", "camera3d.fov_y_degrees", fov);
}

fn hero(world, t) {
  if t < 4.0 { pose(world, "hero-officer", -1.0, 0.0, -8.0, 0.0); }
  else if t < 17.0 { pose(world, "hero-officer", -1.0, 0.0, lerp(-8.0, 8.0, (t - 4.0) / 13.0), 0.0); }
  else if t < 21.0 {
    let u = smoothstep((t - 17.0) / 4.0);
    pose(world, "hero-officer", lerp(-1.0, -4.8, u), 0.0, lerp(8.0, 4.2, u), lerp(0.0, -2.34, u));
  } else {
    let yaw = if t < 26.0 { -2.34 } else { -2.34 + lerp(0.0, 0.13, smoothstep((t - 26.0) / 4.0)) };
    pose(world, "hero-officer", -4.8, 0.0, 4.2, yaw);
  }
}

fn support(world, t) {
  if t < 21.0 { pose(world, "support-officer", 8.75, 0.0, 7.85, -0.45); }
  else if t < 23.2 {
    let u = smoothstep((t - 21.0) / 2.2);
    pose(world, "support-officer", lerp(8.75, 9.15, u), 0.0, lerp(7.85, 7.25, u), -0.20);
  } else { pose(world, "support-officer", 9.15, 0.0, 7.25, -0.20); }
}

fn civilian(world, t) {
  if t < 5.0 { pose(world, "civilian-01", -18.0, 0.0, -10.5, 1.5707963267948966); }
  else if t < 20.0 { pose(world, "civilian-01", lerp(-18.0, 18.0, (t - 5.0) / 15.0), 0.0, -10.5, 1.5707963267948966); }
  else { pose(world, "civilian-01", 18.0, 0.0, -10.5, 1.5707963267948966); }
}

fn traffic(world, t) {
  let police_x = if t < 23.35 { 9.5 } else if t < 26.0 { lerp(9.5, 16.1, (t - 23.35) / 2.65) } else { 16.1 };
  pose(world, "police-car", police_x, 0.0, 5.55, 1.5707963267948966);
  if t < 4.0 { pose(world, "taxi-01", lerp(-32.0, -5.0, t / 4.0), 0.0, -2.75, 1.5707963267948966); }
  else if t >= 10.0 && t <= 12.7 { pose(world, "taxi-01", lerp(-18.0, 18.0, (t - 10.0) / 2.7), 0.0, 2.75, 1.5707963267948966); }
  else { pose(world, "taxi-01", -60.0, 0.0, 2.75, 1.5707963267948966); }
  let sedan_x = if t >= 15.0 && t <= 16.65 { lerp(18.0, -18.0, (t - 15.0) / 1.65) } else { 60.0 };
  pose(world, "sedan-wipe", sedan_x, 0.0, 4.25, -1.5707963267948966);
}

fn hero_x(world) { world.entities.position_3d("hero-officer").x }
fn hero_z(world) { world.entities.position_3d("hero-officer").z }

fn shot(world, t) {
  let hx = hero_x(world); let hz = hero_z(world);
  if t < 4.0 {
    let u = smootherstep(t / 4.0);
    camera(world, lerp(30.0, 22.0, u), lerp(16.5, 10.5, u), lerp(31.0, 23.0, u), 0.0, lerp(1.0, 1.25, u), lerp(1.0, -1.5, u), lerp(43.0, 39.0, u));
  } else if t < 8.0 {
    let u = smoothstep((t - 4.0) / 4.0);
    camera(world, lerp(7.4, 3.9, u), 1.72, lerp(-12.6, -11.1, u), hx, 1.28, hz, 36.0);
  } else if t < 13.0 {
    let u = smoothstep((t - 8.0) / 5.0);
    camera(world, 7.2 + lerp(0.5, -0.35, u), 2.0, hz + 1.15, hx, 1.25, hz, 30.0);
  } else if t < 17.0 {
    if t < 15.72 { camera(world, 7.0, 1.85, hz + 1.0, hx, 1.25, hz, 31.0); }
    else { camera(world, -7.2, 1.55, hz + 0.25, hx, 1.20, hz, 33.0); }
  } else if t < 21.0 {
    let u = smootherstep((t - 17.0) / 4.0);
    camera(world, lerp(4.6, 1.6, u), lerp(0.68, 0.76, u), lerp(9.2, 6.25, u), hx, 1.18, hz, 37.0);
  } else if t < 26.0 {
    let u = smoothstep((t - 21.0) / 5.0);
    let cx = world.entities.position_3d("police-car").x;
    let angle = lerp(-0.70, 0.22, u); let radius = lerp(7.2, 6.2, u);
    let blend = if t < 23.3 { 0.42 } else { 0.12 };
    camera(world, cx + cos_approx(angle) * radius, 2.0, 5.55 + sin_approx(angle) * radius, lerp(cx, 9.15, blend), lerp(0.95, 1.35, blend), lerp(5.55, 7.25, blend), 33.0);
  } else {
    let u = smootherstep((t - 26.0) / 4.0);
    camera(world, lerp(-1.15, -2.05, u), lerp(2.15, 1.95, u), lerp(7.55, 6.35, u), hx, 1.47, hz, lerp(25.0, 22.0, u));
  }
}

fn on_enter() { world.state.set_float("npr-city.start", world.time.elapsed()); }
fn update(dt) {
  let t = timeline_time(world);
  hero(world, t); support(world, t); civilian(world, t); traffic(world, t); shot(world, t);
}
