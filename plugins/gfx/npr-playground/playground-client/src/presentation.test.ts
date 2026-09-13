import { describe, expect, it, vi } from 'vitest';
import type { CompanionHost } from './companion-host';
import type { FrameHeader } from './frames';
import { ViewportPresentation, type PresentationState } from './presentation';

function fixture(nativeError: string | null=null, failRgba=false) {
  const events=new Map<string,(value:any)=>void>(),unlisten=vi.fn();
  const invoke=vi.fn(async()=>undefined),sent: any[]=[];
  const host: CompanionHost={canPickFile:true,nativeError,invoke:invoke as CompanionHost['invoke'],
    async listen(event,callback){events.set(event,callback);return ()=>{unlisten(event);events.delete(event);};}};
  let state: PresentationState | undefined, changed=()=>{}, tick=()=>{}, visible=true;
  let rect={left:100,top:50,width:640,height:360};
  const rgba={push:vi.fn(),dispose:vi.fn()},decoder={push:vi.fn(),dispose:vi.fn()},input=vi.fn();
  const stopMonitor=vi.fn(),stopInterval=vi.fn(),makeRgba=vi.fn(()=>{if(failRgba)throw new Error('RGBA failed');return rgba;});
  const presentation=new ViewportPresentation({viewport:{} as HTMLDivElement,jpeg:{} as HTMLCanvasElement,rgba:{} as HTMLCanvasElement},host,
    {state:value=>state=value,send:message=>{sent.push(message);return true;},input},
    {bounds:()=>rect,dpr:()=>2,visible:()=>visible,monitor:callback=>{changed=callback;return stopMonitor;},interval:callback=>{tick=callback;return stopInterval;},rgba:makeRgba,decoder:()=>decoder});
  const header=(mode:FrameHeader['mode'],generation:number):FrameHeader=>({mode,generation,size:[1280,720],sequence:1,revision:1,input_id:0,format:'rgba8_srgb',stages:{}});
  return {presentation,events,invoke,sent,input,rgba,makeRgba,decoder,stopMonitor,stopInterval,unlisten,header,state:()=>state!,
    move:()=>{rect={...rect,left:140};changed();},hide:()=>{visible=false;changed();},tick:()=>tick()};
}
describe('ViewportPresentation',()=> {
  it('switches generations and hides the native overlay before the first JPEG arrives',async()=> {
    const f=fixture();f.presentation.select('native_gpu');await f.presentation.start();f.presentation.setConnected(true);
    expect(f.sent.at(-1).request.generation).toBe(1);
    f.events.get('viewport-presented')!(f.header('native_gpu',1));expect(f.state().activeMode).toBe('native_gpu');
    f.presentation.select('jpeg');expect(f.sent.at(-1).request.generation).toBe(2);
    expect(f.invoke.mock.calls.at(-1)).toMatchObject(['viewport_rect',{rect:{visible:false}}]);
    f.events.get('viewport-presented')!(f.header('native_gpu',1));expect(f.state().activeMode).toBeNull();
    f.events.get('viewport-error')!({generation:1,message:'obsolete'});expect(f.state().error).toBe('');
    f.events.get('viewport-input')!({kind:'down'});expect(f.input).not.toHaveBeenCalled();
  });
  it('tracks position/visibility even without resizing and hides on disconnect',async()=> {
    const f=fixture();f.presentation.select('native_gpu');await f.presentation.start();f.presentation.setConnected(true);f.move();
    expect(f.invoke.mock.calls.at(-1)).toMatchObject(['viewport_rect',{rect:{left:140,visible:true}}]);
    expect(f.sent.at(-1).request.generation).toBe(1);
    f.hide();expect(f.invoke.mock.calls.at(-1)).toMatchObject(['viewport_rect',{rect:{visible:false}}]);
    f.presentation.setConnected(false);expect(f.state().fps).toBe(0);
  });
  it('falls back to JPEG when Native GPU never presents its startup frame',async()=> {
    const now=vi.spyOn(performance,'now').mockReturnValue(0);
    const f=fixture();f.presentation.select('native_gpu');await f.presentation.start();f.presentation.setConnected(true);
    now.mockReturnValue(4000);
    f.tick();
    expect(f.state().mode).toBe('jpeg');
    expect(f.state().nativeUnavailable).toContain('Native GPU');
    expect(f.sent.at(-1).request.mode).toBe('jpeg');
    now.mockRestore();
  });
  it('retries a lost JPEG startup request',async()=> {
    const now=vi.spyOn(performance,'now').mockReturnValue(0);
    const f=fixture();await f.presentation.start();f.presentation.setConnected(true);
    const initial=f.sent.length;
    now.mockReturnValue(4000);
    f.tick();
    expect(f.sent.length).toBeGreaterThan(initial);
    expect(f.sent.at(-1).request.mode).toBe('jpeg');
    now.mockRestore();
  });
  it('initializes RGBA lazily and an RGBA failure leaves Native GPU available',async()=> {
    const f=fixture(null,true);await f.presentation.start();f.presentation.setConnected(true);
    expect(f.makeRgba).not.toHaveBeenCalled();
    expect(f.presentation.select('local_rgba')).toBe(false);
    expect(f.state().nativeUnavailable).toBeNull();expect(f.state().mode).toBe('jpeg');
    expect(f.state().rgbaUnavailable).toContain('RGBA failed');
  });
  it('does not initialize native APIs in JPEG-only hosts and releases pending native slots',async()=> {
    const f=fixture('native channel unavailable');await f.presentation.start();f.presentation.setConnected(true);
    expect(f.state().mode).toBe('jpeg');expect(f.events.size).toBe(0);expect(f.invoke).not.toHaveBeenCalled();
    const native=fixture();await native.presentation.start();native.presentation.setConnected(true);
    native.events.get('viewport-rgba-frame')!({slot:0,header:native.header('local_rgba',0)});
    expect(native.invoke.mock.calls.at(-1)).toMatchObject(['frame_consumed',{ack:{generation:0,presented:false}}]);
    native.presentation.dispose();native.presentation.dispose();
    expect(native.unlisten).toHaveBeenCalledTimes(4);expect(native.stopMonitor).toHaveBeenCalledOnce();expect(native.decoder.dispose).toHaveBeenCalledOnce();
  });
  it('cleans listeners that register after disposal',async()=> {
    const listeners: ((stop:()=>void)=>void)[]=[],stop=vi.fn();
    const host:CompanionHost={canPickFile:true,nativeError:null,invoke:async<T>()=>undefined as T,listen:()=>new Promise(resolve=>listeners.push(resolve))};
    const presentation=new ViewportPresentation({viewport:{} as HTMLDivElement,jpeg:{} as HTMLCanvasElement,rgba:{} as HTMLCanvasElement},host,{state:()=>{},send:()=>true,input:()=>{}},
      {bounds:()=>({left:0,top:0,width:640,height:360}),dpr:()=>1,visible:()=>true,monitor:()=>()=>{},interval:()=>()=>{},decoder:()=>({push:()=>{},dispose:()=>{}})});
    const started=presentation.start();presentation.dispose();listeners.forEach(resolve=>resolve(stop));await started;
    expect(stop).toHaveBeenCalledTimes(4);
  });
});
