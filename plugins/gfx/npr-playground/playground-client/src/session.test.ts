import { afterEach, describe, expect, it, vi } from 'vitest';
import { applyDelta } from './session';
import { LatestFrameDecoder, type FrameHeader } from './frames';
import { MutationQueue } from './mutation-queue';

afterEach(() => vi.unstubAllGlobals());
describe('session state', () => {
  it('updates telemetry at the same revision without releasing a pending mutation', () => {
    const queue = new MutationQueue();
    queue.push('viewport',{kind:'navigate',mode:'orbit',dx:10,dy:0,wheel:0,x:0,y:0,width:640,height:360,focused:true});
    const action=queue.take(2)!;
    expect(queue.accepted(action.request_id,3,2)).toBe(false);
    const snapshot=applyDelta({revision:2,values:{diagnostics:{fps:20}},metadata:null},
      {base_revision:2,revision:2,changed:{diagnostics:{fps:60}},removed:[]});
    expect(snapshot.values.diagnostics?.fps).toBe(60);
    expect(queue.state(snapshot.revision)).toBe(false);
    expect(queue.busy).toBe(true);
    expect(queue.state(3)).toBe(true);
  });
  it('acknowledges a failed old decode without failing the new surface', async () => {
    let reject: (reason: unknown) => void = () => {};
    vi.stubGlobal('createImageBitmap', () => new Promise((_, fail) => { reject=fail; }));
    const ack=vi.fn(), error=vi.fn(), draw=vi.fn();
    let generation=1;
    const decoder=new LatestFrameDecoder(draw,error,ack,header=>header.generation===generation);
    const header={sequence:1,generation:1,size:[1,1],mode:'jpeg',format:'rgba8_srgb',revision:0,input_id:0,stages:{}} as FrameHeader;
    decoder.push({image:new Blob(['invalid']),header});
    generation=2; reject(new Error('old JPEG decode failed')); await Promise.resolve();
    expect(ack).toHaveBeenCalledWith(expect.objectContaining({sequence:1,generation:1,presented:false}));
    expect(error).not.toHaveBeenCalled();expect(draw).not.toHaveBeenCalled();
    decoder.dispose();
  });
  it('applies a delta without mutating the baseline and rejects missing revisions', () => {
    const snapshot = { revision: 2, values: { keep: 1, remove: 2 }, metadata: 'controls' };
    expect(applyDelta(snapshot, { base_revision: 2, revision: 3, changed: { add: 3 }, removed: ['remove'] }))
      .toEqual({ revision: 3, values: { keep: 1, add: 3 }, metadata: 'controls' });
    expect(snapshot.values.remove).toBe(2);
  expect(() => applyDelta(snapshot, { base_revision: 1, revision: 3, changed: {}, removed: [] })).toThrow('revision');
  });
  it('drops superseded frames, closes decoded images, and keeps only the latest pending frame', async () => {
    const pending: ((value: any) => void)[] = [];
    const decode = vi.fn(() => new Promise(resolve => pending.push(resolve)));
    vi.stubGlobal('createImageBitmap', decode);
    let present: FrameRequestCallback | undefined;
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {present=callback;return 1;});
    vi.stubGlobal('cancelAnimationFrame', vi.fn());
    const ack = vi.fn();
    const draw = vi.fn(); const error = vi.fn(); const decoder = new LatestFrameDecoder(draw, error, ack, () => true);
    const frames = ['a','b','c'].map((text, sequence) => ({ image: new Blob([text]), header: { sequence, generation: 1, size: [1,1], mode:'jpeg',format:'rgba8_srgb',revision:0,input_id:0,stages:{} } as FrameHeader }));
    frames.forEach(frame => decoder.push(frame));
    expect(decode).toHaveBeenCalledTimes(1);
    const first = { width:1,height:1,close: vi.fn() }; pending.shift()!(first); await Promise.resolve();
    expect(draw).not.toHaveBeenCalled();
    // Ready frames may present while a newer frame is decoding: no starvation.
    present!(0); expect(first.close).toHaveBeenCalledOnce();
    expect(decode).toHaveBeenLastCalledWith(frames[2].image);
    const latest = { width:1,height:1,close: vi.fn() }; pending.shift()!(latest); await Promise.resolve();
    present!(0);
    expect(draw).toHaveBeenLastCalledWith(latest,frames[2].header); expect(latest.close).toHaveBeenCalledOnce();
    // A late metadata parse must never reintroduce an older image.
    decoder.push(frames[0]);expect(decode).toHaveBeenCalledTimes(2);
    decoder.push({...frames[0],header:{...frames[0].header,sequence:3}}); decoder.dispose();
    const disposed = { width:1,height:1,close: vi.fn() }; pending.shift()!(disposed); await Promise.resolve();
    expect(draw).toHaveBeenCalledTimes(2); expect(disposed.close).toHaveBeenCalledOnce(); expect(error).not.toHaveBeenCalled();
  });
});
