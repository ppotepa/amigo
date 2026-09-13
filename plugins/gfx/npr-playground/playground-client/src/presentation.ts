import type { NativeViewportInput } from './camera-input';
import type { CompanionHost } from './companion-host';
import type { PresentationMode } from './contracts';
import { LatestFrameDecoder, parseFrame, type EncodedFrame, type FrameAck, type FrameHeader } from './frames';
import { LocalRgbaPresenter } from './rgba';

type Rect = Pick<DOMRect, 'left' | 'top' | 'width' | 'height'>;
interface Elements { viewport: HTMLDivElement; jpeg: HTMLCanvasElement; rgba: HTMLCanvasElement }
interface RgbaPresenter { push(slot: number, header: FrameHeader): void; dispose(): void }
interface Decoder { push(frame: EncodedFrame): void; dispose(): void }
export interface PresentationState {
  mode: PresentationMode; activeMode: PresentationMode | null; size: [number, number]; fps: number;
  error: string; nativeUnavailable: string | null; rgbaUnavailable: string | null;
}
interface Hooks {
  state: (state: PresentationState) => void;
  send: (message: unknown) => boolean;
  input: (input: NativeViewportInput) => void;
}
interface Platform {
  bounds: () => Rect; dpr: () => number; visible: () => boolean;
  monitor: (changed: () => void) => () => void;
  interval: (callback: () => void) => () => void;
  rgba: (current: (h: FrameHeader) => boolean, presented: (h: FrameHeader) => void, ack: (a: FrameAck) => void, error: (e: string) => void) => RgbaPresenter;
  decoder: (draw: (image: ImageBitmap, h: FrameHeader) => void, error: (e: string) => void, ack: (a: FrameAck) => void, current: (h: FrameHeader) => boolean) => Decoder;
}

const NATIVE_STARTUP_TIMEOUT_MS = 4000;

/** One presentation owner for generations, frame credits, capabilities and native bounds. */
export class ViewportPresentation {
  private state: PresentationState;
  private generation = 0;
  private requestKey = '';
  private rectKey = '';
  private connected = false;
  private disposed = false;
  private started = false;
  private frames = 0;
  private frameWaitStarted = -1;
  private nativeWaitStarted = -1;
  private unlisten: (() => void)[] = [];
  private rgba: RgbaPresenter | undefined;
  private decoder: Decoder;
  private platform: Platform;
  constructor(private elements: Elements, private host: CompanionHost, private hooks: Hooks, platform: Partial<Platform> = {}) {
    // JPEG is the deterministic startup path. Native GPU remains available
    // as an explicit opt-in after the workspace has a usable first frame.
    this.state = {mode:'jpeg',activeMode:null,size:[1,1],fps:0,
      error:'',nativeUnavailable:host.nativeError,rgbaUnavailable:host.nativeError};
    this.platform = {
      bounds: () => elements.viewport.getBoundingClientRect(), dpr: () => devicePixelRatio, visible: () => !document.hidden,
      monitor: changed => {
        const observer=new ResizeObserver(changed);observer.observe(elements.viewport);
        window.addEventListener('resize',changed);document.addEventListener('scroll',changed,true);document.addEventListener('visibilitychange',changed);
        return () => {observer.disconnect();window.removeEventListener('resize',changed);document.removeEventListener('scroll',changed,true);document.removeEventListener('visibilitychange',changed);};
      },
      interval: callback => {const timer=window.setInterval(callback,500);return () => window.clearInterval(timer);},
      rgba: (current,presented,ack,error) => new LocalRgbaPresenter(elements.rgba,current,presented,ack,error),
      decoder: (draw,error,ack,current) => new LatestFrameDecoder(draw,error,ack,current),
      ...platform,
    };
    this.decoder=this.platform.decoder((image,header) => {
      const canvas=this.elements.jpeg;
      const context=canvas.getContext('2d');if(!context)throw new Error('Brak kontekstu canvas dla JPEG.');
      canvas.width=image.width;canvas.height=image.height;context.drawImage(image,0,0);this.present(header);
    },message=>this.fail(message),ack=>this.hooks.send({type:'frame_ack',ack}),header=>this.current(header));
    this.publish();
  }
  private publish(patch: Partial<PresentationState> = {}) {
    if(this.disposed)return;
    this.state={...this.state,...patch};this.hooks.state(this.state);
  }
  private current(header: FrameHeader) {return !this.disposed && header.generation===this.generation && header.mode===this.state.mode;}
  async start() {
    if(this.started || this.disposed)return;this.started=true;
    this.unlisten.push(this.platform.monitor(()=>this.request()));
    this.unlisten.push(this.platform.interval(()=>{
      this.publish({fps:this.frames*2});
      this.frames=0;
      if (this.connected && this.state.mode !== 'native_gpu' && !this.state.activeMode
        && this.frameWaitStarted >= 0 && performance.now() - this.frameWaitStarted >= NATIVE_STARTUP_TIMEOUT_MS) {
        this.frameWaitStarted = performance.now();
        this.request();
      }
      if (this.connected && this.state.mode === 'native_gpu' && !this.state.activeMode
        && this.nativeWaitStarted >= 0 && performance.now() - this.nativeWaitStarted >= NATIVE_STARTUP_TIMEOUT_MS) {
        const message = 'Native GPU nie dostarczyło klatki startowej. Przełączono na JPEG.';
        this.publish({nativeUnavailable:message});
        this.select('jpeg');
      }
    }));
    if(this.host.nativeError)return;
    const registered=await Promise.allSettled([
      this.host.listen<FrameHeader>('viewport-presented',header=>this.present(header)),
      this.host.listen<{slot:number;header:FrameHeader}>('viewport-rgba-frame',frame=> {
        if(this.rgba)this.rgba.push(frame.slot,frame.header);
        else this.nativeAck({sequence:frame.header.sequence,generation:frame.header.generation,presented:false,decode_ms:0,present_ms:0});
      }),
      this.host.listen<{generation:number;message:string}>('viewport-error',error=>this.fail(error.message,error.generation)),
      this.host.listen<NativeViewportInput>('viewport-input',input=>{if(!this.disposed && this.state.mode==='native_gpu')this.hooks.input(input);}),
    ]);
    const listeners=registered.flatMap(result=>result.status==='fulfilled'?[result.value]:[]);
    const error=registered.find(result=>result.status==='rejected');
    if(this.disposed || error) {
      listeners.forEach(stop=>stop());
      if(error) {const message=String(error.reason);this.publish({nativeUnavailable:message,rgbaUnavailable:message});this.fail(message);}
    } else this.unlisten.push(...listeners);
  }
  setConnected(connected: boolean) {
    if(this.disposed)return;
    this.connected=connected;
    if(!connected) {this.frames=0;this.publish({fps:0});}
    this.request();
  }
  select(mode: PresentationMode): boolean {
    if(this.disposed)return false;
    const unavailable=mode==='native_gpu'?this.state.nativeUnavailable:mode==='local_rgba'?this.state.rgbaUnavailable:null;
    if(unavailable){this.publish({error:unavailable});return false;}
    if(mode==='local_rgba' && !this.rgba) {
      try {this.rgba=this.platform.rgba(header=>this.current(header),header=>this.present(header),ack=>this.nativeAck(ack),message=>this.fail(message));}
      catch(error) {const message=String(error);this.publish({rgbaUnavailable:message,error:message});return false;}
    }
    if(this.state.mode!==mode) {
      this.frames=0;this.requestKey='';this.publish({mode,activeMode:null,fps:0,error:''});this.request();
    }
    return true;
  }
  request(interacting = false) {
    if(this.disposed)return;
    const rect=this.platform.bounds();
    const dpr=this.platform.dpr();
    if(this.connected && rect.width>0 && rect.height>0) {
      const key=JSON.stringify([rect.width,rect.height,dpr,this.state.mode]);
      if(key!==this.requestKey){
        this.generation++;
        this.requestKey=key;
        this.frameWaitStarted = performance.now();
        if (this.state.mode === 'native_gpu') this.nativeWaitStarted = performance.now();
      }
      this.hooks.send({type:'viewport',request:{mode:this.state.mode,generation:this.generation,adaptive_resolution:false,
        css_width:rect.width,css_height:rect.height,dpr,interacting,high_quality_capture:false}});
    }
    this.syncNative(rect,dpr);
  }
  private syncNative(rect=this.platform.bounds(),dpr=this.platform.dpr()) {
    if(this.host.nativeError)return;
    const value={left:rect.left,top:rect.top,width:rect.width,height:rect.height,dpr,
      visible:!this.disposed && this.connected && this.platform.visible() && rect.width>0 && rect.height>0 && this.state.mode==='native_gpu'};
    const key=JSON.stringify([this.state.mode,value]);if(key===this.rectKey)return;this.rectKey=key;
    const generation=this.generation;
    void this.host.invoke('viewport_rect',{rect:value}).catch(error=>this.fail(String(error),generation));
  }
  private present(header: FrameHeader) {
    if(!this.current(header))return;
    if (header.mode === 'native_gpu') {
      this.nativeWaitStarted = 0;
      // NativeSurface.place() makes the child visible only after its first
      // frame. Re-sync here so a later mode switch cannot leave that child
      // covering the JPEG canvas.
      this.rectKey = '';
      this.syncNative();
    }
    this.frames++;this.publish({activeMode:header.mode,size:header.size});
  }
  private nativeAck(ack: FrameAck) {
    // Old generations also release their slots; filtering acknowledgements leaks credits.
    void this.host.invoke('frame_consumed',{ack}).catch(error=>this.fail(String(error),ack.generation));
  }
  private fail(message: string,generation?:number) {
    if(this.disposed || (generation!==undefined && generation!==this.generation))return;
    if(this.state.mode!=='jpeg'){
      this.publish({nativeUnavailable:message});
      this.select('jpeg');
    }
    this.publish({error:message});
  }
  async jpeg(blob: Blob) {
    try {const frame=await parseFrame(blob);this.decoder.push(frame);}
    catch(error){if(!this.disposed)this.fail(String(error));}
  }
  dispose() {
    if(this.disposed)return;
    this.disposed=true;this.connected=false;this.syncNative();
    this.unlisten.splice(0).forEach(stop=>stop());this.decoder.dispose();this.rgba?.dispose();
  }
}
