import type { NprIntent } from './contracts';

/** One mutation lane. A matching ACK AND its state release the next action. */
export class MutationQueue {
  private queue: { control: string; intent: NprIntent; queued_at: number }[] = [];
  private active: { request: number; revision?: number } | null = null;
  private sequence = 0;
  get busy(): boolean { return this.active !== null || this.queue.length > 0; }
  push(control: string, intent: NprIntent) {
    const last = this.queue.at(-1);
    if (last?.intent.kind === 'preview_layer' && intent.kind === 'preview_layer' && last.intent.layer === intent.layer) {
      last.intent = intent;
      return;
    }
    if (last?.intent.kind === 'navigate' && intent.kind === 'navigate'
      && ['orbit', 'pan', 'zoom'].includes(intent.mode)
      && last.intent.mode === intent.mode && last.intent.width === intent.width && last.intent.height === intent.height) {
      last.intent.dx += intent.dx;
      last.intent.dy += intent.dy;
      last.intent.wheel += intent.wheel;
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
