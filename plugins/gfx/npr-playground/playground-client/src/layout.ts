export type PanelSide = 'left' | 'right';

export function clampPanelWidth(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, Number.isFinite(value) ? value : minimum));
}

export function panelWidthFromDelta(side: PanelSide, start: number, delta: number, minimum: number, maximum: number): number {
  return clampPanelWidth(start + (side === 'left' ? delta : -delta), minimum, maximum);
}

export function panelWidthFromKey(side: PanelSide, key: string, value: number, minimum: number, maximum: number, step = 16): number | undefined {
  const direction = side === 'left' ? 1 : -1;
  if (key === 'ArrowLeft') return clampPanelWidth(value - direction * step, minimum, maximum);
  if (key === 'ArrowRight') return clampPanelWidth(value + direction * step, minimum, maximum);
  if (key === 'Home') return minimum;
  if (key === 'End') return maximum;
  return undefined;
}
