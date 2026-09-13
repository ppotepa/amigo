import type { CompanionBootstrap, NprIntent, PlaygroundValues } from './contracts';
import { MutationQueue } from './mutation-queue';

export type Snapshot = { revision: number; values: PlaygroundValues; metadata: unknown };
export type Delta = { base_revision: number; revision: number; changed: Partial<PlaygroundValues>; removed: string[]; metadata?: unknown };
export function applyDelta(snapshot: Snapshot, delta: Delta): Snapshot {
  if (snapshot.revision !== delta.base_revision) throw new Error('State revision mismatch; resynchronizing session.');
  const values = { ...snapshot.values, ...delta.changed };
  for (const key of delta.removed) delete values[key];
  return { revision: delta.revision, values, metadata: delta.metadata ?? snapshot.metadata };
}

type WireMessage =
  | { type: 'snapshot'; snapshot: Snapshot }
  | ({ type: 'delta' } & Delta)
  | { type: 'accepted'; request_id: number; revision: number }
  | { type: 'rejected'; request_id: number; error: { code: string; message: string } }
  | { type: 'domain'; name: string; payload: unknown };
export interface SessionState {
  snapshot: Snapshot | null;
  connection: 'connecting' | 'connected' | 'resyncing' | 'disconnected';
  pending: boolean;
  error: string;
}
interface SessionHooks {
  state: (state: SessionState) => void;
  frame: (blob: Blob) => void;
  connected?: () => void;
  disconnected?: () => void;
}
interface SessionPlatform {
  socket: (url: string) => WebSocket;
  schedule: (callback: () => void) => number;
  cancel: (id: number) => void;
}

/** The sole WebSocket, snapshot and mutation owner. Never retries authored actions. */
export class DrawingSession {
  private socket: WebSocket | undefined;
  private closedSocket: WebSocket | undefined;
  private boot: CompanionBootstrap | undefined;
  private reconnectTimer: ReturnType<typeof setTimeout> | undefined;
  private mutations = new MutationQueue();
  private scheduled: number | undefined;
  private disposed = false;
  private current: SessionState = { snapshot: null, connection: 'connecting', pending: false, error: '' };
  constructor(private hooks: SessionHooks, private platform: SessionPlatform = {
    socket: url => new WebSocket(url), schedule: callback => requestAnimationFrame(callback), cancel: id => cancelAnimationFrame(id),
  }) {}
  get ready() { return this.current.connection === 'connected' && this.socket?.readyState === 1 && !!this.current.snapshot; }
  private publish(patch: Partial<SessionState> = {}) {
    if (this.disposed) return;
    this.current = { ...this.current, ...patch, pending: this.mutations.busy };
    this.hooks.state(this.current);
  }
  connect(boot: CompanionBootstrap) {
    if (this.disposed || this.socket) return;
    this.boot = boot;
    try {
      const socket = this.socket = this.platform.socket(boot.endpoint);
      socket.binaryType = 'blob';
      socket.onopen = () => {
        if (!this.disposed) this.send({ version: boot.version, playground: boot.playground, token: boot.token });
      };
      socket.onmessage = event => {
        if (this.disposed) return;
        if (event.data instanceof Blob) { this.hooks.frame(event.data); return; }
        try { this.receive(JSON.parse(String(event.data)) as WireMessage); }
        catch (error) { this.publish({ error: `Błąd protokołu: ${String(error)}` }); this.refresh(); }
      };
      socket.onclose = () => {
        if (this.socket === socket) { this.socket = undefined; this.closedSocket = socket; }
        this.disconnected();
        this.scheduleReconnect();
      };
      socket.onerror = () => {
        this.disconnected('Błąd połączenia. Otwórz Drawing Studio ponownie ze sceny.');
        socket.close();
        this.scheduleReconnect();
      };
      this.publish();
    } catch (error) { this.disconnected(String(error)); this.scheduleReconnect(); }
    }
  private scheduleReconnect() {
    if (this.disposed || this.socket || this.reconnectTimer !== undefined || !this.boot) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = undefined;
      if (!this.disposed && this.boot) this.connect(this.boot);
    }, 500);
  }
  send(message: unknown): boolean {
    if (this.disposed || this.socket?.readyState !== 1) return false;
    try { this.socket.send(JSON.stringify(message)); return true; }
    catch (error) { this.disconnected(String(error)); return false; }
  }
  dispatch(control: string, intent: NprIntent): boolean {
    if (!this.ready) return false;
    this.mutations.push(control, intent);
    this.publish({ error: '' });
    this.schedule();
    return true;
  }
  refresh() {
    if (this.current.connection === 'resyncing') return;
    if (this.send({ type: 'refresh' })) this.publish({ connection: 'resyncing' });
  }
  private receive(message: WireMessage) {
    if (message.type === 'snapshot') {
      if (!Number.isSafeInteger(message.snapshot?.revision) || !message.snapshot.values) throw new Error('Invalid snapshot');
      const connected = this.current.connection !== 'connected';
      this.mutations.state(message.snapshot.revision);
      this.publish({ snapshot: message.snapshot, connection: 'connected' });
      if (connected) this.hooks.connected?.();
      this.schedule();
    } else if (message.type === 'delta') {
      if (this.current.connection === 'resyncing') return;
      if (!this.current.snapshot || this.current.snapshot.revision !== message.base_revision) { this.refresh(); return; }
      const snapshot = applyDelta(this.current.snapshot, message);
      this.mutations.state(snapshot.revision);
      this.publish({ snapshot });
      this.schedule();
    } else if (message.type === 'accepted') {
      this.mutations.accepted(message.request_id, message.revision, this.current.snapshot?.revision ?? 0);
      this.publish(); this.schedule();
    } else if (message.type === 'rejected') {
      if (!this.mutations.rejected(message.request_id)) return;
      this.publish({ error: `${message.error.code}: ${message.error.message}` });
      if (message.error.code === 'revision_conflict') this.refresh();
    }
  }
  private schedule() {
    if (!this.ready || this.scheduled !== undefined) return;
    this.scheduled = this.platform.schedule(() => {
      this.scheduled = undefined;
      if (!this.ready) return;
      const action = this.mutations.take(this.current.snapshot!.revision);
      if (action) this.send({ type: 'action', action });
      this.publish();
    });
  }
  private disconnected(error = this.current.error || 'Połączenie przerwane. Otwórz Drawing Studio ponownie ze sceny.') {
    if (this.disposed || this.current.connection === 'disconnected') return;
    this.mutations.clear();
    if (this.scheduled !== undefined) this.platform.cancel(this.scheduled);
    this.scheduled = undefined;
    this.publish({ connection: 'disconnected', error });
    this.hooks.disconnected?.();
  }
  dispose() {
    if (this.disposed) return;
    this.disposed = true;
    if (this.reconnectTimer !== undefined) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = undefined;
    if (this.scheduled !== undefined) this.platform.cancel(this.scheduled);
    this.mutations.clear();
    const socket = this.socket ?? this.closedSocket;
    if (socket) {
      socket.onopen = socket.onmessage = socket.onclose = socket.onerror = null;
      socket.close();
    }
    this.socket = undefined;
    this.closedSocket = undefined;
  }
}
