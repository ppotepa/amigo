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

  const fast = (keys.has('ShiftLeft') || keys.has('ShiftRight')) ? 2.8 : 1;
  const move = 1.45 * fast * dt / Math.max(0.45, state.zoom);
  const yaw = deg(state.cameraYaw), pitch = deg(state.cameraPitch);
  const forwardAxis = v3(Math.sin(yaw)*Math.cos(pitch), -Math.sin(pitch), Math.cos(yaw)*Math.cos(pitch));
  const rightAxis = v3(Math.cos(yaw), 0, -Math.sin(yaw));
  const upAxis = v3(Math.sin(yaw)*Math.sin(pitch), Math.cos(pitch), Math.cos(yaw)*Math.sin(pitch));

  state.cameraX = clamp(state.cameraX + (rightAxis.x*right + upAxis.x*up + forwardAxis.x*forward) * move, -3, 3);
  state.cameraY = clamp(state.cameraY + (rightAxis.y*right + upAxis.y*up + forwardAxis.y*forward) * move, -3, 3);
  state.cameraZ = clamp(state.cameraZ + (rightAxis.z*right + upAxis.z*up + forwardAxis.z*forward) * move, -3, 3);
  setCameraAngles(state, (state.rawCameraYaw ?? state.cameraYaw) + lookX * 72 * dt * fast, (state.rawCameraPitch ?? state.cameraPitch) + lookY * 60 * dt * fast);
  state.controlMode = 'freelook';
  state.auto = false;
  return true;
}
