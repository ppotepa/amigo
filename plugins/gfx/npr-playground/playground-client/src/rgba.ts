import type { FrameAck, FrameHeader } from './frames';

type SharedEvent = { getBuffer(): ArrayBuffer; additionalData: { generation: number; slot: number; size: [number, number] } };

/** Persistent WebGL texture fed directly from read-only WebView2 SharedBuffers. */
export class LocalRgbaPresenter {
  private buffers = new Map<string, ArrayBuffer>();
  private waiting: { slot: number; header: FrameHeader } | null = null;
  private frame: number | undefined;
  private gl: WebGL2RenderingContext;
  private texture: WebGLTexture;
  private program: WebGLProgram;
  private size = '';
  private latest: FrameHeader | undefined;
  private invalid = false;
  private generation = -1;
  get lostContext() {return this.invalid || this.gl.isContextLost();}
  private webview = (window as any).chrome?.webview;
  constructor(private canvas: HTMLCanvasElement, private current: (header: FrameHeader) => boolean,
    private presented: (header: FrameHeader) => void, private ack: (ack: FrameAck) => void, private error: (message: string) => void) {
    const gl = canvas.getContext('webgl2', { alpha: false, antialias: false, depth: false });
    if (!gl || !this.webview) throw new Error('Local RGBA requires WebGL2 and WebView2 SharedBuffer');
    if(gl.isContextLost())throw new Error('Local RGBA WebGL device is still lost');
    this.gl = gl;
    const shader = (kind: number, source: string) => {
      const shader = gl.createShader(kind)!; gl.shaderSource(shader, source); gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {const message=gl.getShaderInfoLog(shader);gl.deleteShader(shader);throw new Error(message ?? 'RGBA shader compilation failed');}
      return shader;
    };
    const vertex = shader(gl.VERTEX_SHADER, `#version 300 es
      out vec2 uv; void main() { vec2 p=vec2((gl_VertexID<<1)&2,gl_VertexID&2); uv=vec2(p.x,1.0-p.y); gl_Position=vec4(p*2.0-1.0,0,1); }`);
    const fragment = shader(gl.FRAGMENT_SHADER, `#version 300 es
      precision highp float; in vec2 uv; uniform sampler2D pixels; out vec4 color;
      void main() { color=texture(pixels,uv); }`);
    this.program=gl.createProgram()!; gl.attachShader(this.program,vertex); gl.attachShader(this.program,fragment); gl.linkProgram(this.program);
    gl.deleteShader(vertex);gl.deleteShader(fragment);
    if(!gl.getProgramParameter(this.program,gl.LINK_STATUS))throw new Error(gl.getProgramInfoLog(this.program) ?? 'RGBA shader link failed');
    gl.useProgram(this.program); this.texture=gl.createTexture()!;gl.bindTexture(gl.TEXTURE_2D,this.texture);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);
    // Source bytes are already sRGB encoded, just like the JPEG canvas output.
    gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL,gl.NONE);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL,false);
    this.webview.addEventListener('sharedbufferreceived',this.receive);
    this.canvas.addEventListener('webglcontextlost',this.lost);
  }
  private lost=(event:Event)=>{event.preventDefault();this.invalid=true;if(this.latest&&this.current(this.latest))this.error('Local RGBA WebGL device lost. Select another presentation mode.');};
  private receive = (event: SharedEvent) => {
    const data=event.additionalData;const buffer=event.getBuffer();
    if(data.generation<this.generation){this.webview.releaseBuffer(buffer);return;}
    this.generation=data.generation;
    for(const [key,old] of this.buffers)if(!key.startsWith(`${data.generation}:`)){this.webview.releaseBuffer(old);this.buffers.delete(key);}
    const key=`${data.generation}:${data.slot}`;
    const old=this.buffers.get(key);if(old)this.webview.releaseBuffer(old);
    this.buffers.set(key,buffer); this.schedule();
  };
  private acknowledge(header: FrameHeader, presented=false, present_ms=0) {
    this.ack({sequence:header.sequence,generation:header.generation,presented,decode_ms:0,present_ms});
  }
  push(slot:number, header:FrameHeader) {
    if(!this.current(header)){this.acknowledge(header);return;}
    this.latest=header;
    if(this.waiting)this.acknowledge(this.waiting.header);
    this.waiting={slot,header};this.schedule();
  }
  private schedule() {if(this.waiting && this.frame===undefined)this.frame=requestAnimationFrame(this.draw);}
  private draw=()=>{
    this.frame=undefined;const waiting=this.waiting;if(!waiting)return;
    if(!this.current(waiting.header)){this.waiting=null;this.acknowledge(waiting.header);return;}
    const pixels=this.buffers.get(`${waiting.header.generation}:${waiting.slot}`);if(!pixels)return;
    this.waiting=null;const start=performance.now();let presented=false;
    try {
      const gl=this.gl;const [width,height]=waiting.header.size;
      if(pixels.byteLength!==width*height*4)throw new Error('RGBA shared buffer dimensions disagree with frame');
      if(this.canvas.width!==width)this.canvas.width=width;if(this.canvas.height!==height)this.canvas.height=height;
      gl.viewport(0,0,width,height);gl.bindTexture(gl.TEXTURE_2D,this.texture);
      const size=`${width}:${height}`;
      if(this.size!==size){gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA8,width,height,0,gl.RGBA,gl.UNSIGNED_BYTE,null);this.size=size;}
      gl.texSubImage2D(gl.TEXTURE_2D,0,0,0,width,height,gl.RGBA,gl.UNSIGNED_BYTE,new Uint8Array(pixels));
      gl.useProgram(this.program);gl.drawArrays(gl.TRIANGLES,0,3);gl.flush();
      if(gl.isContextLost())throw new Error('Local RGBA WebGL device lost');
      this.presented(waiting.header);presented=true;
    }catch(error){this.error(String(error));}
    finally {this.acknowledge(waiting.header,presented,performance.now()-start);}
  };
  dispose(){
    if(this.frame!==undefined)cancelAnimationFrame(this.frame);
    if(this.waiting)this.acknowledge(this.waiting.header);
    this.webview.removeEventListener('sharedbufferreceived',this.receive);
    this.canvas.removeEventListener('webglcontextlost',this.lost);
    for(const buffer of this.buffers.values())this.webview.releaseBuffer(buffer);
    this.buffers.clear();this.gl.deleteTexture(this.texture);this.gl.deleteProgram(this.program);
  }
}
