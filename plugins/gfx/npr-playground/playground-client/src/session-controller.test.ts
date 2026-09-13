import { describe, expect, it, vi } from 'vitest';
import { DrawingSession, type SessionState } from './session';

function fixture() {
  const sent: Record<string, any>[] = [];
  const socket = { readyState: 0, binaryType: '', onopen: null, onmessage: null, onclose: null, onerror: null,
    send: (value: string) => sent.push(JSON.parse(value)), close: vi.fn(),
  } as unknown as WebSocket;
  const scheduled = new Map<number, () => void>(); let sequence = 0;
  let state: SessionState | undefined;
  const connected=vi.fn(), disconnected=vi.fn(), frame=vi.fn();
  const session = new DrawingSession({state: value => state=value,connected,disconnected,frame}, {
    socket: () => socket, schedule: callback => { scheduled.set(++sequence,callback);return sequence; }, cancel: id => { scheduled.delete(id); },
  });
  const message = (value: unknown) => socket.onmessage?.call(socket,{data:JSON.stringify(value)} as MessageEvent);
  session.connect({endpoint:'ws://127.0.0.1:1234',version:1,playground:'npr-playground',token:'test-only'});
  Object.defineProperty(socket,'readyState',{value:1,writable:true});
  socket.onopen?.call(socket,new Event('open'));
  const snapshot = (revision=2) => message({type:'snapshot',snapshot:{revision,values:{diagnostics:{fps:1}},metadata:null}});
  const flush = () => {const callbacks=[...scheduled.values()];scheduled.clear();callbacks.forEach(fn=>fn());};
  return {session,socket,sent,message,snapshot,flush,connected,disconnected,frame,state:()=>state!};
}
describe('DrawingSession', () => {
  it('authenticates before actions and requires both the ACK and state', () => {
    const f=fixture();expect(f.sent[0]).toEqual({version:1,playground:'npr-playground',token:'test-only'});
    expect(f.session.dispatch('save',{kind:'save_all'})).toBe(false);
    f.snapshot();expect(f.connected).toHaveBeenCalledOnce();
    f.session.dispatch('save',{kind:'save_all'});f.session.dispatch('undo',{kind:'undo'});f.flush();
    expect(f.sent.filter(v=>v.type==='action')).toHaveLength(1);
    const action=f.sent.at(-1)!.action;
    f.message({type:'accepted',request_id:action.request_id,revision:3});f.flush();
    expect(f.sent.filter(v=>v.type==='action')).toHaveLength(1);
    f.message({type:'delta',base_revision:2,revision:2,changed:{diagnostics:{fps:60}},removed:[]});f.flush();
    expect(f.state().pending).toBe(true);
    f.message({type:'delta',base_revision:2,revision:3,changed:{},removed:[]});f.flush();
    expect(f.sent.at(-1)!.action.intent.kind).toBe('undo');expect(f.sent.at(-1)!.action.base_revision).toBe(3);
  });
  it('resynchronizes revision conflicts and never replays rejected authored actions', () => {
    const f=fixture();f.snapshot();f.session.dispatch('save',{kind:'save_all'});f.flush();
    const action=f.sent.at(-1)!.action;f.session.dispatch('undo',{kind:'undo'});
    f.message({type:'rejected',request_id:action.request_id,error:{code:'revision_conflict',message:'stale'}});
    expect(f.state().connection).toBe('resyncing');expect(f.sent.at(-1)!.type).toBe('refresh');
    expect(f.session.dispatch('save',{kind:'save_all'})).toBe(false);
    f.snapshot(9);f.flush();expect(f.state().pending).toBe(false);
    expect(f.sent.filter(v=>v.type==='action')).toHaveLength(1);
    expect(f.session.dispatch('undo',{kind:'undo'})).toBe(true);f.flush();expect(f.sent.at(-1)!.action.base_revision).toBe(9);
  });
  it('requests one refresh for a delta gap and accepts same-revision telemetry', () => {
    const f=fixture();f.snapshot();
    f.message({type:'delta',base_revision:7,revision:8,changed:{},removed:[]});
    f.message({type:'delta',base_revision:8,revision:9,changed:{},removed:[]});
    expect(f.sent.filter(v=>v.type==='refresh')).toHaveLength(1);
    f.snapshot(9);f.message({type:'delta',base_revision:9,revision:9,changed:{diagnostics:{fps:60}},removed:[]});
    expect(f.state().snapshot?.values.diagnostics?.fps).toBe(60);
  });
  it('forwards frames and cancels scheduled actions/listeners on shutdown', () => {
    const f=fixture();f.snapshot();f.session.dispatch('save',{kind:'save_all'});
    const blob=new Blob(['frame']);f.socket.onmessage?.call(f.socket,{data:blob} as MessageEvent);
    expect(f.frame).toHaveBeenCalledWith(blob);
    f.socket.onclose?.call(f.socket,new Event('close') as CloseEvent);
    expect(f.state().connection).toBe('disconnected');expect(f.state().pending).toBe(false);f.flush();
    expect(f.sent.filter(v=>v.type==='action')).toHaveLength(0);
    f.session.dispose();f.session.dispose();expect(f.socket.close).toHaveBeenCalledOnce();expect(f.socket.onmessage).toBeNull();
  });
});
