import { clamp, clamp01, lerp, noise } from '../math/core.js';

export function computeImpreciseSampleTime(state, step, duration) {
  if (!state.impreciseTween || state.tweenJitterFrames <= 0) {
    state.animJitterFrames = 0;
    return wrapTime(state.animTime, duration);
  }
  const frame = state.animFrameIndex;
  const a = noise(41.73, frame);
  const b = noise(93.17, Math.floor(frame / 3));
  const holdBias = noise(12.31, Math.floor(frame / 5)) * 0.35;
  const targetFrames = clamp((a * 0.72 + b * 0.28 + holdBias) * state.tweenJitterFrames, -state.tweenJitterFrames, state.tweenJitterFrames);
  state.animJitterFrames = lerp(state.animJitterFrames, targetFrames, 0.42);
  return wrapTime(state.animTime + state.animJitterFrames * step, duration);
}

export function randomnessFrameSeed(state) {
  const variance = clamp01(state.contourFrameVariance || 0);
  if (variance <= 0) return 0;
  if (state.modelSource === 'walking') return Math.floor(state.animFrameIndex * variance);
  return Math.floor((state.rawYaw ?? state.yaw) * 0.18 * variance);
}

export function shadowRandomSeed(state) {
  const layout = clamp01(state.shadowLayoutJitter || 0);
  if (layout <= 0) return 0;
  const frameDrift = clamp01(state.shadowFrameDrift || 0);
  const loopRedraw = clamp01(state.shadowLoopRedraw || 0);
  const frameSource = state.modelSource === 'walking'
    ? state.animFrameIndex
    : Math.round((state.rawYaw ?? state.yaw) * 0.25);
  const frameSeed = Math.floor(frameSource * frameDrift);
  const loopSeed = state.modelSource === 'walking'
    ? Math.floor((state.animLoopIndex || 0) * loopRedraw * 97)
    : 0;
  return (frameSeed * 197.31 + loopSeed * 571.19) * layout;
}

function wrapTime(value, duration) {
  if (!duration || duration <= 0) return value;
  let t = value % duration;
  if (t < 0) t += duration;
  return t;
}
