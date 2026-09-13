import { afterEach, describe, expect, it, vi } from 'vitest';
import type { NativeViewportInput } from './camera-input';
import type { NprIntent } from './contracts';
import { ViewportInput } from './viewport-input';

afterEach(()=>vi.useRealTimers());
function fixture() {
  const sent:NprIntent[]=[];let enabled=true;
  const viewport={getBoundingClientRect:()=>({left:10,top:20,width:640,height:360}),clientHeight:360,focus:vi.fn(),setPointerCapture:vi.fn(),hasPointerCapture:()=>false,releasePointerCapture:vi.fn()} as unknown as HTMLDivElement;
  const controller=new ViewportInput(viewport,{enabled:()=>enabled,mode:()=> 'select',size:()=>[1280,720],dispatch:intent=>{sent.push(intent);return enabled;},interacting:vi.fn(),shortcut:vi.fn()});
  const pointer=(x:number,y:number)=>({pointerId:7,button:0,clientX:x,clientY:y,preventDefault:vi.fn()}) as unknown as PointerEvent;
  const native=(kind:NativeViewportInput['kind'],x:number,y:number)=>controller.native({kind,x,y,button:0,wheel:0,key:0});
  return {controller,viewport,sent,pointer,native,disable:()=>enabled=false};
}
describe('ViewportInput',()=> {
  it('includes final mouse-up movement identically for DOM and native drags',()=> {
    const dom=fixture();dom.controller.pointerDown(dom.pointer(10,20));dom.controller.pointerUp(dom.pointer(30,25));
    const native=fixture();native.native('down',10,20);native.native('up',30,25);
    expect(native.sent).toEqual(dom.sent);
    expect(dom.sent).toMatchObject([{kind:'begin_camera_gesture'},{kind:'navigate',mode:'orbit',dx:20,dy:5},{kind:'end_camera_gesture'}]);
  });
  it('maps a click into render coordinates and stops after lost capture',()=> {
    const f=fixture();f.controller.pointerDown(f.pointer(330,200));f.controller.pointerUp(f.pointer(330,200));
    expect(f.sent[1]).toMatchObject({mode:'select',x:640,y:360,width:1280,height:720});
    f.controller.pointerDown(f.pointer(10,20));f.controller.cancel();const count=f.sent.length;
    f.controller.pointerMove(f.pointer(50,60));expect(f.sent).toHaveLength(count);
  });
  it('ends wheel bursts and cancels timers during teardown',()=> {
    vi.useFakeTimers();const f=fixture();
    f.controller.wheel({deltaY:2,deltaMode:1,preventDefault:()=>{}} as WheelEvent);
    expect(f.sent[1]).toMatchObject({mode:'zoom',wheel:-32});
    vi.advanceTimersByTime(180);expect(f.sent.at(-1)?.kind).toBe('end_camera_gesture');
    f.controller.dispose();const count=f.sent.length;vi.runAllTimers();expect(f.sent).toHaveLength(count);
  });
  it('does not start mutations when the session is not ready',()=> {
    const f=fixture();f.disable();f.controller.pointerDown(f.pointer(0,0));f.controller.navigate('fit');f.native('down',0,0);
    expect(f.sent).toHaveLength(0);
  });
});
