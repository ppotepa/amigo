import { CameraGestureController, WheelGestureEnd, nativeShortcut, pickCoordinates, wheelPixels,
  type CameraMode, type NativeViewportInput, type Shortcut } from './camera-input';
import type { NavigationMode, NprIntent } from './contracts';

interface Hooks {
  enabled: () => boolean; mode: () => CameraMode; size: () => [number,number];
  dispatch: (intent: NprIntent) => boolean; interacting: (active: boolean) => void;
  shortcut: (action: Shortcut | null) => void;
}

/** DOM and native input share the same gesture lifetime and coordinate units. */
export class ViewportInput {
  private gestures = new CameraGestureController();
  private wheelEnd = new WheelGestureEnd(() => {this.end();this.hooks.interacting(false);});
  private sequence = 0;
  private activeGesture: number | undefined;
  constructor(private viewport: HTMLDivElement, private hooks: Hooks) {}
  private begin() {
    if(this.activeGesture!==undefined)return;
    const id=++this.sequence;
    if(this.hooks.dispatch({kind:'begin_camera_gesture',gesture_id:id}))this.activeGesture=id;
  }
  private end() {
    if(this.activeGesture===undefined)return;
    this.hooks.dispatch({kind:'end_camera_gesture',gesture_id:this.activeGesture});this.activeGesture=undefined;
  }
  navigate(mode: NavigationMode,dx=0,dy=0,wheel=0,x=0,y=0) {
    if(!this.hooks.enabled())return;
    const rect=this.viewport.getBoundingClientRect(),size=this.hooks.size();
    this.hooks.dispatch({kind:'navigate',mode,dx,dy,wheel,x,y,
      width:mode==='select'?size[0]:Math.max(1,Math.round(rect.width)),
      height:mode==='select'?size[1]:Math.max(1,Math.round(rect.height)),focused:true});
    this.hooks.interacting(true);
  }
  private move(id:number,x:number,y:number) {
    const movement=this.gestures.move(id,x,y);
    if(movement)this.navigate(movement.mode,movement.dx,movement.dy,movement.wheel);
  }
  private down(id:number,button:number,x:number,y:number) {
    if(!this.hooks.enabled() || !this.gestures.down(id,button,x,y,this.hooks.mode()))return false;
    this.wheelEnd.cancel();this.end();this.begin();return true;
  }
  pointerDown=(event:PointerEvent)=> {
    if(!this.down(event.pointerId,event.button,event.clientX,event.clientY))return;
    event.preventDefault();this.viewport.focus();this.viewport.setPointerCapture(event.pointerId);
  };
  pointerMove=(event:PointerEvent)=>this.move(event.pointerId,event.clientX,event.clientY);
  pointerUp=(event:PointerEvent)=> {
    if(!this.gestures.owns(event.pointerId))return;
    this.move(event.pointerId,event.clientX,event.clientY);
    if(this.gestures.up(event.pointerId,event.clientX,event.clientY)) {
      const [x,y]=pickCoordinates(event.clientX,event.clientY,this.viewport.getBoundingClientRect(),this.hooks.size());
      this.navigate('select',0,0,0,x,y);
    }
    this.end();
    if(this.viewport.hasPointerCapture(event.pointerId))this.viewport.releasePointerCapture(event.pointerId);
    this.hooks.interacting(false);
  };
  private zoom(delta:number) {
    if(!this.hooks.enabled())return;
    this.begin();this.navigate('zoom',0,0,delta);if(!this.gestures.active)this.wheelEnd.touch();
  }
  wheel=(event:WheelEvent)=> {
    event.preventDefault();this.viewport.focus();this.zoom(-wheelPixels(event.deltaY,event.deltaMode,this.viewport.clientHeight));
  };
  native=(input:NativeViewportInput)=> {
    if(!this.hooks.enabled())return;
    if(input.kind==='down')this.down(0,input.button,input.x,input.y);
    else if(input.kind==='move')this.move(0,input.x,input.y);
    else if(input.kind==='up' && this.gestures.owns(0)) {
      this.move(0,input.x,input.y);
      if(this.gestures.up(0,input.x,input.y)) {
        const rect=this.viewport.getBoundingClientRect(),size=this.hooks.size();
        this.navigate('select',0,0,0,input.x*size[0]/rect.width,input.y*size[1]/rect.height);
      }
      this.end();this.hooks.interacting(false);
    } else if(input.kind==='wheel')this.zoom(input.wheel);
    else if(input.kind==='key')this.hooks.shortcut(nativeShortcut(input));
    else if(input.kind==='cancel')this.cancel();
  };
  cancel=()=> {
    this.wheelEnd.cancel();const pointer=this.gestures.cancel();this.end();
    if(pointer!==undefined && this.viewport.hasPointerCapture(pointer))this.viewport.releasePointerCapture(pointer);
    this.hooks.interacting(false);
  };
  dispose(){this.cancel();}
}
