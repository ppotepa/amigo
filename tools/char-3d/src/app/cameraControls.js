import { clamp, deg, v3 } from '../math/core.js';

export const cameraKeyCodes = new Set([
  'KeyW',
  'KeyA',
  'KeyS',
  'KeyD',
  'KeyQ',
  'KeyE',
  'ArrowLeft',
  'ArrowRight',
  'ArrowUp',
  'ArrowDown',
  'ShiftLeft',
  'ShiftRight',
]);

export function isTypingTarget(target) {
  const tag = target?.tagName;
  return !!(target?.isContentEditable || tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || tag === 'BUTTON');
}

export function wrapAngle(value) {
  let out = value;
  while (out > 180) out -= 360;
  while (out < -180) out += 360;
  return out;
}

export function snapAngle(value, angleSnap) {
  const step = Number(angleSnap) || 0;
  return step > 0 ? wrapAngle(Math.round(value / step) * step) : wrapAngle(value);
}

export function snapPitch(value, angleSnap) {
  const step = Number(angleSnap) || 0;
  return step > 0 ? clamp(Math.round(value / step) * step, -85, 85) : clamp(value, -85, 85);
}

export function setModelAngles(state, yaw, pitch) {
  state.rawYaw = wrapAngle(yaw);
  state.rawPitch = clamp(pitch, -85, 85);
  state.yaw = snapAngle(state.rawYaw, state.angleSnap);
  state.pitch = snapPitch(state.rawPitch, state.angleSnap);
}

export function setCameraAngles(state, yaw, pitch) {
  state.rawCameraYaw = wrapAngle(yaw);
  state.rawCameraPitch = clamp(pitch, -85, 85);
  state.cameraYaw = snapAngle(state.rawCameraYaw, state.angleSnap);
  state.cameraPitch = snapPitch(state.rawCameraPitch, state.angleSnap);
}

export function applyAngleSnap(state) {
  setModelAngles(state, state.rawYaw ?? state.yaw, state.rawPitch ?? state.pitch);
  setCameraAngles(state, state.rawCameraYaw ?? state.cameraYaw, state.rawCameraPitch ?? state.cameraPitch);
}

export function cameraDollyScale(state) {
  return Math.exp(clamp(state.cameraZ, -3, 3) * 0.22);
}

export function updateCameraFromKeys(state, dt) {
  if (state.controlMode !== 'freelook') return false;
  const keys = state.pressedKeys;
  if (!keys.size || dt <= 0) return false;
  
  let right = 0, up = 0, forward = 0, lookX = 0, lookY = 0;
  if (keys.has('KeyD')) right += 1;
  if (keys.has('KeyA')) right -= 1;
  if (keys.has('KeyE')) up += 1;
  if (keys.has('KeyQ')) up -= 1;
  if (keys.has('KeyW')) forward += 1;
  if (keys.has('KeyS')) forward -= 1;
  if (keys.has('ArrowRight')) lookX += 1;
  if (keys.has('ArrowLeft')) lookX -= 1;
  if (keys.has('ArrowDown')) lookY += 1;
  if (keys.has('ArrowUp')) lookY -= 1;
  
  if (!right && !up && !forward && !lookX && !lookY) return false;

  const fast = (keys.has('ShiftLeft') || keys.has('ShiftRight')) ? 3.0 : 1;
  const moveSpeed = 8.0 * fast * dt;
  
  const yawRad = deg(state.cameraYaw);
  const pitchRad = deg(state.cameraPitch);
  
  // Forward axis (world space direction camera is looking)
  // Looking at -Z when Yaw=0, Pitch=0
  const fwd = v3(
    Math.sin(yawRad) * Math.cos(pitchRad),
    -Math.sin(pitchRad),
    -Math.cos(yawRad) * Math.cos(pitchRad)
  );
  
  // Right axis (perpendicular to Forward and World Up)
  const rgt = v3(Math.cos(yawRad), 0, Math.sin(yawRad));
  
  // World Up
  const vup = v3(0, 1, 0);

  state.cameraX += (rgt.x * right + fwd.x * forward) * moveSpeed;
  state.cameraY += (rgt.y * right + fwd.y * forward + vup.y * up) * moveSpeed;
  state.cameraZ += (rgt.z * right + fwd.z * forward) * moveSpeed;
  
  setCameraAngles(state, 
    state.cameraYaw + lookX * 100 * dt * fast, 
    state.cameraPitch + lookY * 100 * dt * fast
  );
  
  return true;
}
