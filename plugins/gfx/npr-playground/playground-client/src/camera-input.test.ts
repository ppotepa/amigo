import { describe, expect, it, vi } from 'vitest';
import { isClick, CameraGestureController, WheelGestureEnd, nativeShortcut, workspaceShortcut, resolvePointerMode, zoomFromDrag, pickCoordinates, wheelPixels } from './camera-input';
import { MutationQueue } from './mutation-queue';

describe('camera input mapping', () => {
  it('closes each wheel burst and cancels the pending end when a drag takes over', () => {
    vi.useFakeTimers();
    try {
      const finish = vi.fn(), wheel = new WheelGestureEnd(finish);
      wheel.touch(); vi.advanceTimersByTime(100); wheel.touch();
      vi.advanceTimersByTime(179); expect(finish).not.toHaveBeenCalled();
      vi.advanceTimersByTime(1); expect(finish).toHaveBeenCalledTimes(1);
      wheel.touch(); wheel.cancel(); vi.advanceTimersByTime(200);
      expect(finish).toHaveBeenCalledTimes(1);
      wheel.touch(); vi.advanceTimersByTime(180);
      expect(finish).toHaveBeenCalledTimes(2);
    } finally { vi.useRealTimers(); }
  });
  it('maps native keys and preserves button activation and text editing', () => {
    const input = {kind:'key' as const,x:0,y:0,button:0,wheel:0,key:32};
    expect(nativeShortcut(input)).toBe('play');
    expect(nativeShortcut({...input,repeat:true})).toBeNull();
    expect(nativeShortcut({...input,key:90,ctrl:true,shift:true})).toBe('redo');
    expect(nativeShortcut({...input,key:83,ctrl:true})).toBe('save');
    expect(nativeShortcut({...input,key:79})).toBe('orbit');
    expect(nativeShortcut({...input,key:86})).toBe('select');
    expect(nativeShortcut({...input,key:27})).toBe('cancel');
    expect(nativeShortcut({...input,alt:true})).toBeNull();
    expect(workspaceShortcut({key:' '},false,true)).toBeNull();
    expect(workspaceShortcut({key:'o'},true)).toBeNull();
    expect(workspaceShortcut({key:'z',ctrl:true},true)).toBeNull();
    expect(workspaceShortcut({key:'s',ctrl:true},true)).toBe('save');
  });
  it('picks the rendered pixel at DPR 1/2 and normalizes wheel units',()=>{
    const css={left:20,top:40,width:640,height:360};
    expect(pickCoordinates(340,220,css,[640,360])).toEqual([320,180]);
    expect(pickCoordinates(340,220,css,[1280,720])).toEqual([640,360]);
    expect(wheelPixels(3,0,360)).toBe(3);
    expect(wheelPixels(3,1,360)).toBe(48);
    expect(wheelPixels(1,2,360)).toBe(360);
  });
  it('keeps separate gesture boundaries and discards rejected relative input',()=>{
    const queue=new MutationQueue();
    for(const id of [1,2]){
      queue.push('viewport',{kind:'begin_camera_gesture',gesture_id:id});
      queue.push('viewport',{kind:'navigate',mode:'zoom',dx:0,dy:0,wheel:3,width:640,height:360,x:0,y:0,focused:true});
      queue.push('viewport',{kind:'navigate',mode:'zoom',dx:0,dy:0,wheel:5,width:640,height:360,x:0,y:0,focused:true});
      queue.push('viewport',{kind:'end_camera_gesture',gesture_id:id});
    }
    const begin=queue.take(0)!;queue.accepted(begin.request_id,0,0);
    const move=queue.take(0)!;expect(move.intent).toMatchObject({kind:'navigate',wheel:8});
    queue.accepted(move.request_id,1,1);
    expect(queue.take(1)?.intent.kind).toBe('end_camera_gesture');
    queue.rejected(3);expect(queue.take(2)).toBeNull();
  });
  it('keeps the standard secondary-button camera gestures', () => {
    expect(resolvePointerMode(2, 'select')).toBe('orbit');
    expect(resolvePointerMode(1, 'select')).toBe('pan');
  });

  it('uses the selected mode for the primary drag', () => {
    expect(resolvePointerMode(0, 'orbit')).toBe('orbit');
    expect(resolvePointerMode(0, 'pan')).toBe('pan');
    expect(resolvePointerMode(0, 'zoom')).toBe('zoom');
    expect(resolvePointerMode(0, 'select')).toBe('select');
  });

  it('orbits on a primary drag while retaining click-to-select', () => {
    const gesture = new CameraGestureController();
    gesture.down(1, 0, 10, 10, 'select');
    expect(gesture.move(1, 20, 14)).toEqual({ mode: 'orbit', dx: 10, dy: 4, wheel: 0 });
    expect(gesture.up(1, 20, 14)).toBe(false);
    gesture.down(2, 0, 10, 10, 'select');
    expect(gesture.up(2, 10, 10)).toBe(true);
  });

  it('distinguishes a click from a drag and maps vertical zoom motion', () => {
    expect(isClick(2, 2)).toBe(true);
    expect(isClick(4, 0)).toBe(false);
    expect(zoomFromDrag(10)).toBe(-10);
    expect(zoomFromDrag(-10)).toBe(10);
  });

  it('accumulates relative movement and waits for the matching ACK and state', () => {
    const queue = new MutationQueue();
    queue.push('save', {kind: 'save_all'});
    const first = queue.take(5)!;
    for (const dx of [1, 2, 3]) queue.push('viewport', {kind:'navigate',mode:'orbit',dx,dy:0,wheel:0,width:640,height:360,x:0,y:0,focused:true});
    expect(queue.accepted(first.request_id + 1, 6, 6)).toBe(false);
    expect(queue.state(6)).toBe(false);
    expect(queue.take(6)).toBeNull();
    expect(queue.accepted(first.request_id, 7, 6)).toBe(false);
    expect(queue.state(7)).toBe(true);
    const move = queue.take(7)!;
    expect(move.intent).toMatchObject({kind:'navigate',dx:6});
    queue.push('viewport', {kind:'navigate',mode:'zoom',wheel:12,dx:0,dy:0,width:640,height:360,x:0,y:0,focused:true});
    expect(queue.rejected(move.request_id)).toBe(true);
    expect(queue.take(8)).toBeNull();
  });
  it('crosses the drag threshold once and keeps all subsequent small movements', () => {
    const gesture = new CameraGestureController();
    gesture.down(1, 2, 10, 10, 'select');
    expect(gesture.move(1, 12, 10)).toBeNull();
    expect(gesture.move(1, 15, 10)?.dx).toBe(5);
    expect(gesture.move(1, 16, 10)?.dx).toBe(1);
    expect(gesture.move(2, 100, 100)).toBeNull();
    expect(gesture.up(1, 16, 10)).toBe(false);
    gesture.down(1, 0, 0, 0, 'select');
    gesture.move(1, 10, 0);
    expect(gesture.up(1, 0, 0)).toBe(false);
    gesture.down(1, 1, 0, 0, 'select');
    expect(gesture.cancel()).toBe(1);
    expect(gesture.move(1, 10, 0)).toBeNull();
  });
});
