export type CameraMode = 'select' | 'orbit' | 'pan' | 'zoom';

export type Intent = Record<string, unknown>;

export function resolvePointerMode(button: number, mode: CameraMode): CameraMode {
  if (button === 2) return 'orbit';
  if (button === 1) return 'pan';
  return mode;
}

export function isClick(dx: number, dy: number, threshold = 4): boolean {
  return dx * dx + dy * dy < threshold * threshold;
}

export function zoomFromDrag(dy: number): number {
  return -dy;
}

export function wheelPixels(delta: number, mode: number, height: number): number {
  return delta * (mode === 1 ? 16 : mode === 2 ? height : 1);
}

export function pickCoordinates(x: number, y: number, css: {left:number;top:number;width:number;height:number}, render: [number,number]): [number,number] {
  return [(x-css.left)*render[0]/css.width,(y-css.top)*render[1]/css.height];
}

/** Canvas and native events use CSS coordinates and the same gesture state. */
export class CameraGestureController {
  get active(): boolean { return this.pointer !== null; }
  owns(id: number): boolean { return this.pointer?.id === id; }
  private pointer: { id: number; x: number; y: number; startX: number; startY: number; mode: CameraMode; dragging: boolean } | null = null;
  down(id: number, button: number, x: number, y: number, mode: CameraMode): boolean {
    if (this.pointer || button > 2) return false;
    this.pointer = { id, x, y, startX: x, startY: y, mode: resolvePointerMode(button, mode), dragging: false };
    return true;
  }
  move(id: number, x: number, y: number): { mode: CameraMode; dx: number; dy: number; wheel: number } | null {
    const p = this.pointer;
    if (!p || p.id !== id) return null;
    if (!p.dragging && isClick(x - p.startX, y - p.startY)) return null;
    p.dragging = true;
    const dx = x - p.x, dy = y - p.y;
    p.x = x; p.y = y;
    if (p.mode === 'select') return null;
    return { mode: p.mode, dx: p.mode === 'zoom' ? 0 : dx, dy: p.mode === 'zoom' ? 0 : dy, wheel: p.mode === 'zoom' ? zoomFromDrag(dy) : 0 };
  }
  up(id: number, x: number, y: number): boolean {
    const p = this.pointer;
    if (!p || p.id !== id) return false;
    this.pointer = null;
    return p.mode === 'select' && !p.dragging && isClick(x - p.startX, y - p.startY);
  }
  cancel(): number | undefined { const id = this.pointer?.id; this.pointer = null; return id; }
}

/** One domain mutation lane. Only a matching ACK and its state release it. */
export class MutationQueue {
  private queue: { control: string; intent: Intent; queued_at: number }[] = [];
  private active: { request: number; revision?: number } | null = null;
  private sequence = 0;
  get busy(): boolean { return this.active !== null || this.queue.length > 0; }
  push(control: string, intent: Intent) {
    const last = this.queue.at(-1);
    if (last?.intent.kind === 'navigate' && intent.kind === 'navigate'
      && ['orbit', 'pan', 'zoom'].includes(String(intent.mode))
      && last.intent.mode === intent.mode && last.intent.width === intent.width && last.intent.height === intent.height) {
      for (const axis of ['dx', 'dy', 'wheel']) last.intent[axis] = Number(last.intent[axis]) + Number(intent[axis]);
    } else this.queue.push({ control, intent, queued_at: performance.now() });
  }
  take(revision: number) {
    if (this.active) return null;
    const next = this.queue.shift();
    if (!next) return null;
    this.active = { request: ++this.sequence };
    return { ...next, request_id: this.sequence, base_revision: revision };
  }
  accepted(request: number, revision: number, currentRevision: number): boolean {
    if (this.active?.request !== request) return false;
    this.active.revision = revision;
    return this.state(currentRevision);
  }
  state(revision: number): boolean {
    if (this.active?.revision === undefined || revision < this.active.revision) return false;
    this.active = null;
    return true;
  }
  rejected(request: number): boolean {
    if (this.active?.request !== request) return false;
    this.clear();
    return true;
  }
  clear() { this.queue = []; this.active = null; }
}
