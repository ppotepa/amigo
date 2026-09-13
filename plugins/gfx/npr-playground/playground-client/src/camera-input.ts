export type CameraMode = 'select' | 'orbit' | 'pan' | 'zoom';

export type Shortcut = 'save' | 'undo' | 'redo' | 'cancel' | 'play' | 'orbit' | 'select';
type ShortcutKey = { key: string; ctrl?: boolean; shift?: boolean; alt?: boolean; meta?: boolean; repeat?: boolean };
export type NativeViewportInput = Omit<ShortcutKey, 'key'> & {
  kind: 'down' | 'up' | 'move' | 'wheel' | 'cancel' | 'focus' | 'key';
  x: number; y: number; button: number; wheel: number; key: number;
};

/** Both input transports obey the same shortcuts and leave controls' keys alone. */
export function workspaceShortcut(input: ShortcutKey, editing = false, interactive = false): Shortcut | null {
  if (input.repeat || input.alt) return null;
  const key = input.key.toLowerCase();
  if (input.ctrl || input.meta) {
    if (key === 's') return 'save';
    if (!editing && key === 'z') return input.shift ? 'redo' : 'undo';
    return null;
  }
  if (key === 'escape') return 'cancel';
  if (editing || interactive) return null;
  return key === ' ' ? 'play' : key === 'o' ? 'orbit' : key === 'v' ? 'select' : null;
}
export function nativeShortcut(input: NativeViewportInput): Shortcut | null {
  return workspaceShortcut({ ...input, key: input.key === 27 ? 'Escape' : String.fromCharCode(input.key) });
}

/** A wheel burst is one undo gesture, terminated after input goes idle. */
export class WheelGestureEnd {
  private timer: ReturnType<typeof setTimeout> | undefined;
  constructor(private readonly finish: () => void) {}
  touch() {
    this.cancel();
    this.timer = setTimeout(() => { this.timer = undefined; this.finish(); }, 180);
  }
  cancel() { clearTimeout(this.timer); this.timer = undefined; }
}

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
    const mode = p.mode === 'select' ? 'orbit' : p.mode;
    return { mode, dx: mode === 'zoom' ? 0 : dx, dy: mode === 'zoom' ? 0 : dy, wheel: mode === 'zoom' ? zoomFromDrag(dy) : 0 };
  }
  up(id: number, x: number, y: number): boolean {
    const p = this.pointer;
    if (!p || p.id !== id) return false;
    this.pointer = null;
    return p.mode === 'select' && !p.dragging && isClick(x - p.startX, y - p.startY);
  }
  cancel(): number | undefined { const id = this.pointer?.id; this.pointer = null; return id; }
}
