import { describe, expect, it } from 'vitest';
import { clampPanelWidth, panelWidthFromDelta, panelWidthFromKey } from './layout';

describe('panel layout', () => {
  it('clamps pointer resizing at both limits', () => {
    expect(panelWidthFromDelta('left', 240, -1000, 200, 360)).toBe(200);
    expect(panelWidthFromDelta('right', 380, 1000, 340, 560)).toBe(340);
    expect(panelWidthFromDelta('left', 240, 40, 200, 360)).toBe(280);
  });

  it('uses edge-aware keyboard directions', () => {
    expect(panelWidthFromKey('left', 'ArrowRight', 240, 200, 360)).toBe(256);
    expect(panelWidthFromKey('right', 'ArrowRight', 380, 340, 560)).toBe(364);
    expect(panelWidthFromKey('left', 'Home', 240, 200, 360)).toBe(200);
    expect(panelWidthFromKey('right', 'End', 380, 340, 560)).toBe(560);
    expect(panelWidthFromKey('left', 'Escape', 240, 200, 360)).toBeUndefined();
    expect(clampPanelWidth(Number.NaN, 200, 360)).toBe(200);
  });
});
