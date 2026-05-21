import { clamp01, lerp } from './core.js';

export function escapeHtml(value) {
  return String(value).replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch] || ch));
}

export function normalizeHexColor(value, fallback = '#000000') {
  const s = String(value || '').trim();
  if (/^#[0-9a-f]{6}$/i.test(s)) return s;
  return fallback;
}

export function hexRgb(hex, fallback = '#000000') {
  const value = normalizeHexColor(hex, fallback).slice(1);
  return {
    r: parseInt(value.slice(0, 2), 16),
    g: parseInt(value.slice(2, 4), 16),
    b: parseInt(value.slice(4, 6), 16),
  };
}

export function mixRgb(a, b, t) {
  return {
    r: Math.round(lerp(a.r, b.r, t)),
    g: Math.round(lerp(a.g, b.g, t)),
    b: Math.round(lerp(a.b, b.b, t)),
  };
}

export function rgba(rgb, alpha) {
  return `rgba(${rgb.r},${rgb.g},${rgb.b},${clamp01(alpha)})`;
}
