(()=>{
'use strict';
const BASE_IMAGE_URL = 'pngs/base-image.png';
const DEPTH_IMAGE_URL = 'pngs/depth-image.png';
const SURFACE_IMAGE_URL = 'pngs/surface-image.png';
const AUX_IMAGE_URL = 'pngs/aux-image.png';
const MATERIAL_IMAGE_URL = 'pngs/material-image.png';
const EMISSIVE_IMAGE_URL = 'pngs/emissive-image.png';
const W = 1672, H = 941, MAX_LIGHTS = 16;
const $ = s => document.querySelector(s);
const clamp = (v,a,b)=>Math.min(b,Math.max(a,v));
const lerp = (a,b,t)=>a+(b-a)*t;
const smooth = (a,b,x)=>{ const t=clamp((x-a)/(b-a||1),0,1); return t*t*(3.0-2.0*t); };
const lum = c => c[0]*0.2126 + c[1]*0.7152 + c[2]*0.0722;
const hex = rgb => '#'+rgb.map(v=>Math.round(clamp(v,0,1)*255).toString(16).padStart(2,'0')).join('');
const hexRgb = h => { const x=h.replace('#',''); return [parseInt(x.slice(0,2),16)/255, parseInt(x.slice(2,4),16)/255, parseInt(x.slice(4,6),16)/255]; };
const uid = ()=>'L'+Math.random().toString(36).slice(2,9);
const TOOL_INFO = {
  moveLight:'<b>Move Light</b><br>Click/drag wybrane światło.<br>Wheel = Z-depth.<br>Shift+wheel = radius/size.<br>Alt+wheel = intensity.',
  rotateLight:'<b>Rotate Direction</b><br>Drag końcówkę strzałki wybranego światła spot/directional.<br>R = ten tryb, Shift = precyzja.',
  focus:'<b>Focus</b><br>Kliknij punkt ostrości.<br>Wheel = focus depth.<br>Shift+wheel = aperture.<br>Alt+wheel = bokeh gain.',
  probe:'<b>Probe</b><br>Kliknij piksel aby podejrzeć wartości map i wynikowego pipeline’u.',
  pan:'<b>Pan / Zoom</b><br>Drag = pan.<br>Wheel = zoom.',
  surfReflect:'<b>Paint Reflectivity</b><br>Surface R channel. Więcej odbicia/specular.',
  surfRough:'<b>Paint Roughness</b><br>Surface G channel. Wyżej = bardziej matowo.',
  surfGlass:'<b>Paint Transmission/Glass</b><br>Surface B channel. Wyżej = bardziej szkło/transmisja.',
  surfMask:'<b>Paint Surface Mask</b><br>Surface A channel. Ogranicza działanie materiału.',
  auxDepth:'<b>Paint Depth</b><br>DepthAux R / pomocniczy depth.',
  auxHeight:'<b>Paint Height</b><br>DepthAux G / local height.',
  auxOcc:'<b>Paint Occluder</b><br>DepthAux B / siła rzucania cienia.',
  auxMask:'<b>Paint Valid Mask</b><br>DepthAux A / uczestnictwo w efektach.',
  paintMaterial:'<b>Paint Material ID</b><br>Maluj semantyczny materiał/obiekt: szkło, drewno, plastik, papier itd.'
};
const BRUSH_PRESETS = {
  matte:{label:'Matte', reflect:0.10, rough:0.88, glass:0.02, mask:1, depth:0.55, height:0.04, occ:0.15, auxMask:1},
  polishedWood:{label:'Polished wood', reflect:0.42, rough:0.38, glass:0.02, mask:1, depth:0.22, height:0.12, occ:0.55, auxMask:1},
  glass:{label:'Glass', reflect:0.92, rough:0.08, glass:0.95, mask:1, depth:0.16, height:0.30, occ:0.60, auxMask:1},
  bottle:{label:'Bottle glass', reflect:0.84, rough:0.12, glass:0.86, mask:1, depth:0.17, height:0.62, occ:0.92, auxMask:1},
  paper:{label:'Paper', reflect:0.06, rough:0.96, glass:0.00, mask:1, depth:0.11, height:0.07, occ:0.22, auxMask:1},
  plastic:{label:'Plastic', reflect:0.33, rough:0.40, glass:0.03, mask:1, depth:0.12, height:0.10, occ:0.55, auxMask:1},
  metal:{label:'Metal', reflect:0.88, rough:0.18, glass:0.06, mask:1, depth:0.18, height:0.18, occ:0.72, auxMask:1}
};
const VIEW_OPTIONS = [
  'final','base','depth','depthAux','effectiveDepth','surfaceReflect','surfaceRough','surfaceGlass','surfaceMask','materialID','emissive','normal','shadow','selectedShadow','glassMask','specular','bokehSource','coc'
];
const STATE = {
  tool:'moveLight',
  viewMode:'final',
  quality:'balanced',
  dirty:true,
  selectedLightId:null,
  status:'',
  probe:null,
  drag:null,
  viewport:{zoom:1, panX:0, panY:0},
  brush:{size:30, hardness:0.82, strength:0.7, preset:'glass', materialId:2},
  render:{
    renderScale:0.72,
    ambient:0.12,
    platePreserve:0.30,
    relightBlend:0.78,
    baseDarkness:0.48,
    albedoGain:1.05,
    computedLightGain:1.10,
    shadowLift:0.18,
    highlightSuppress:0.22,
    exposure:1.03,
    contrast:1.06,
    saturation:0.98,
    heightScale:0.22,
    normalStrength:2.3,
    aoStrength:0.28,
    reflectionStrength:0.45,
    specularBoost:1.1,
    glassTransmission:0.82,
    glassRefraction:0.48,
    glassFresnel:1.10,
    glassEdge:0.85,
    glassTint:[0.97,0.99,1.00],
    materialInfluence:0.0,
    shadowStrength:0.75,
    shadowBias:0.028,
    shadowSoftness:0.35,
    shadowSteps:20,
    focusDepth:0.22,
    aperture:0.10,
    nearBlur:2.2,
    farBlur:1.5,
    bokehThreshold:0.88,
    bokehGain:0.95,
    bloomGain:0.18,
    grain:0.016,
    vignette:0.14,
    anamorphic:1.0,
    selectedShadowOnly:false,
    dofEnabled:true,
    useBokeh:true,
    liveShadows:true,
    showHandles:true
  },
  lights:[]
};
const PRESETS = {
  noirDefault(){
    Object.assign(STATE.render,{ambient:0.12,platePreserve:0.30,relightBlend:0.78,baseDarkness:0.48,albedoGain:1.05,computedLightGain:1.10,shadowLift:0.18,highlightSuppress:0.22,exposure:1.03,contrast:1.06,saturation:0.98,heightScale:0.22,normalStrength:2.3,aoStrength:0.28,reflectionStrength:0.45,specularBoost:1.1,glassTransmission:0.82,glassRefraction:0.48,glassFresnel:1.10,glassEdge:0.85,materialInfluence:0.0,shadowStrength:0.75,shadowBias:0.028,shadowSoftness:0.35,shadowSteps:20,focusDepth:0.22,aperture:0.10,nearBlur:2.2,farBlur:1.5,bokehThreshold:0.88,bokehGain:0.95,bloomGain:0.18,grain:0.016,vignette:0.14,anamorphic:1.0,selectedShadowOnly:false,dofEnabled:true,useBokeh:true,liveShadows:true,showHandles:true});
    STATE.lights = [
      {id:uid(),name:'Desk Lamp',type:'spot',x:0.744,y:0.463,z:0.28,radius:0.55,intensity:2.40,color:[1.00,0.73,0.42],dirX:-0.43,dirY:0.48,cone:0.82,shadow:0.84,spec:1.35,bokeh:1.15,enabled:true,width:0.22,height:0.18},
      {id:uid(),name:'Window Area',type:'area',x:0.575,y:0.286,z:0.92,radius:0.40,intensity:1.24,color:[0.57,0.68,1.00],dirX:0,dirY:1,cone:1.0,shadow:0.42,spec:0.72,bokeh:1.25,enabled:true,width:0.26,height:0.34},
      {id:uid(),name:'TV Glow',type:'emissive',x:0.211,y:0.405,z:0.56,radius:0.28,intensity:0.62,color:[0.38,0.76,1.00],dirX:0,dirY:1,cone:1.0,shadow:0.0,spec:0.42,bokeh:0.55,enabled:true,width:0.16,height:0.12},
      {id:uid(),name:'Bottle Kicker',type:'point',x:0.464,y:0.537,z:0.18,radius:0.16,intensity:0.95,color:[1.0,0.83,0.58],dirX:0,dirY:0,cone:1.0,shadow:0.0,spec:1.85,bokeh:0.75,enabled:true,width:0.1,height:0.1}
    ];
    STATE.selectedLightId = STATE.lights[0].id;
  },
  deskLampFocus(){ PRESETS.noirDefault(); const w = findLight('Window Area'); if(w){w.intensity=0.78;} const d=findLight('Desk Lamp'); if(d){d.intensity=2.9; d.shadow=0.88;} Object.assign(STATE.render,{focusDepth:0.20,aperture:0.085,bokehGain:0.85,reflectionStrength:0.52}); },
  coldWindow(){ PRESETS.noirDefault(); const d=findLight('Desk Lamp'); if(d){d.intensity=1.15;} const w=findLight('Window Area'); if(w){w.intensity=1.65; w.shadow=0.50;} Object.assign(STATE.render,{ambient:0.10,focusDepth:0.44,aperture:0.12,bokehThreshold:0.84,bokehGain:1.15,reflectionStrength:0.34}); },
  tvMood(){ PRESETS.noirDefault(); const tv=findLight('TV Glow'); if(tv){tv.intensity=1.05; tv.bokeh=0.7;} const d=findLight('Desk Lamp'); if(d){d.intensity=1.7;} Object.assign(STATE.render,{ambient:0.09,platePreserve:0.42,relightBlend:0.72,bloomGain:0.24,focusDepth:0.28,aperture:0.14}); },
  previewSoft(){ PRESETS.noirDefault(); Object.assign(STATE.render,{ambient:0.16,platePreserve:0.44,relightBlend:0.62,shadowStrength:0.52,shadowSoftness:0.48,dofEnabled:false,useBokeh:false,reflectionStrength:0.28}); }
};
const QUALITY = {
  draft:{renderScale:0.50,shadowSteps:12},
  balanced:{renderScale:0.72,shadowSteps:20},
  high:{renderScale:0.86,shadowSteps:28},
  ultra:{renderScale:1.00,shadowSteps:36}
};
function findLight(name){ return STATE.lights.find(l=>l.name===name) || null; }
const el = {
  toolbar: $('#toolbar'), sidebar: $('#sidebar'), stage: $('#stage'), help: $('#helpBox'), status: $('#status'), badge: $('#renderBadge'),
  glCanvas: $('#glView'), overlay: $('#overlay')
};
const ctx2d = el.overlay.getContext('2d');
const assets = {base:null,depth:null,surface:null,aux:null,material:null,emissive:null};
const maps = {baseCanvas:null,depthCanvas:null,surfaceCanvas:null,auxCanvas:null, baseCtx:null, depthCtx:null, surfaceCtx:null, auxCtx:null};
let gl, vao, litProgram, postProgram, litFbo, litTex, depthRb, uniformsLit={}, uniformsPost={}, textures={};
let renderW=0, renderH=0, raf=0, lastTs=0, fps=0;
const fullVs = `#version 300 es
precision highp float;
out vec2 v_uv;
void main(){
  vec2 p = vec2((gl_VertexID == 1) ? 3.0 : -1.0, (gl_VertexID == 2) ? 3.0 : -1.0);
  v_uv = p * 0.5 + 0.5;
  gl_Position = vec4(p, 0.0, 1.0);
}`;
const litFs = `#version 300 es
precision highp float;
#define MAX_LIGHTS ${MAX_LIGHTS}
in vec2 v_uv; out vec4 outColor;
uniform sampler2D u_base; uniform sampler2D u_depth; uniform sampler2D u_surface; uniform sampler2D u_aux; uniform sampler2D u_material; uniform sampler2D u_emissive;
uniform vec2 u_texel; uniform int u_debugMode; uniform int u_selectedLightIndex; uniform bool u_selectedShadowOnly;
uniform float u_ambient; uniform float u_platePreserve; uniform float u_relightBlend; uniform float u_baseDarkness; uniform float u_albedoGain; uniform float u_computedLightGain; uniform float u_shadowLift; uniform float u_highlightSuppress; uniform float u_heightScale; uniform float u_normalStrength; uniform float u_aoStrength; uniform float u_reflectionStrength; uniform float u_specularBoost; uniform float u_glassTransmission; uniform float u_glassRefraction; uniform float u_glassFresnel; uniform float u_glassEdge; uniform vec3 u_glassTint; uniform float u_materialInfluence; uniform float u_shadowStrength; uniform float u_shadowBias; uniform float u_shadowSoftness; uniform int u_shadowSteps; uniform bool u_liveShadows;
uniform int u_lightCount; uniform vec4 u_lightPosRad[MAX_LIGHTS]; uniform vec4 u_lightColorInt[MAX_LIGHTS]; uniform vec4 u_lightDirType[MAX_LIGHTS]; uniform vec4 u_lightExtra[MAX_LIGHTS];
float luma(vec3 c){ return dot(c, vec3(0.2126,0.7152,0.0722)); }
float matIs(float id,float target){ return 1.0 - step(0.5, abs(id-target)); }
float sampleDepthMap(vec2 uv){ return texture(u_depth, uv).r; }
vec4 sampleAux(vec2 uv){ return texture(u_aux, uv); }
float sampleEffDepth(vec2 uv){ vec4 a=sampleAux(uv); return clamp(sampleDepthMap(uv)-a.g*u_heightScale*a.a,0.0,1.0); }
vec3 normalAt(vec2 uv){ float dl=sampleEffDepth(uv-vec2(u_texel.x,0.0)); float dr=sampleEffDepth(uv+vec2(u_texel.x,0.0)); float dt=sampleEffDepth(uv-vec2(0.0,u_texel.y)); float db=sampleEffDepth(uv+vec2(0.0,u_texel.y)); return normalize(vec3((dl-dr)*u_normalStrength,(dt-db)*u_normalStrength,1.0)); }
float aoAt(vec2 uv,float d0){ float occ=0.0; float cnt=0.0; for(int y=-2;y<=2;y++){ for(int x=-2;x<=2;x++){ if(x==0&&y==0) continue; float sd=sampleEffDepth(uv+vec2(float(x),float(y))*u_texel*2.0); occ += max(0.0,d0-sd); cnt += 1.0; }} return clamp(1.0-occ/max(cnt,1.0)*8.0*u_aoStrength,0.0,1.0); }
float shadowRay(vec2 uv,float d0,int idx){ if(!u_liveShadows) return 1.0; if(u_lightExtra[idx].z < 0.5) return 1.0; float type=u_lightDirType[idx].w; vec2 target=u_lightPosRad[idx].xy; float lightZ=u_lightPosRad[idx].z; vec2 dir2; float dist2d; if(type>2.5&&type<3.5){ vec2 d=normalize(vec2(-u_lightDirType[idx].x,-u_lightDirType[idx].y)); dir2=d; dist2d=0.50; target=uv+d*dist2d; lightZ=1.0; } else { dir2=target-uv; dist2d=length(dir2); dir2/=max(dist2d,0.0001); }
  float blocked=0.0; float steps=float(u_shadowSteps); for(int s=1;s<64;s++){ if(float(s)>steps) break; float t=float(s)/(steps+1.0); vec2 suv=uv+dir2*dist2d*t; vec4 a=sampleAux(suv); float expected=mix(d0,lightZ,t); float sd=sampleEffDepth(suv); float pen=max(0.0, expected-sd-u_shadowBias); blocked += step(0.0005,pen)*a.b*a.a; }
  blocked=blocked/max(steps,1.0); float st=mix(u_shadowStrength,u_lightExtra[idx].x,0.65); return clamp(1.0-blocked*st*(1.0-0.35*u_shadowSoftness),0.02,1.0); }
void main(){ vec2 uv=v_uv; vec3 baseRaw=texture(u_base,uv).rgb; vec4 aux=texture(u_aux,uv); vec4 surf=texture(u_surface,uv); vec4 em=texture(u_emissive,uv); float mid=floor(texture(u_material,uv).r*15.0+0.5); float effD=sampleEffDepth(uv); vec3 n=normalAt(uv); float ao=aoAt(uv,effD);
  vec3 lifted=mix(baseRaw,sqrt(max(baseRaw,0.0)),u_shadowLift); vec3 compressed=lifted/(1.0+lifted*u_highlightSuppress*1.6); vec3 base=mix(lifted,compressed,u_highlightSuppress); vec3 plate=baseRaw*u_baseDarkness*u_platePreserve; vec3 albedo=base*u_baseDarkness*u_albedoGain;
  float reflectivity=surf.r; float roughness=clamp(surf.g,0.02,1.0); float glassness=surf.b; float mask=surf.a;
  float mWood=matIs(mid,1.0), mGlass=matIs(mid,2.0)+matIs(mid,3.0)+matIs(mid,5.0)+matIs(mid,11.0), mLiquid=matIs(mid,4.0), mPlastic=matIs(mid,6.0), mMetal=matIs(mid,7.0)+matIs(mid,12.0), mPaper=matIs(mid,8.0), mTV=matIs(mid,9.0), mCurtain=matIs(mid,10.0);
  mWood*=u_materialInfluence; mGlass*=u_materialInfluence; mLiquid*=u_materialInfluence; mPlastic*=u_materialInfluence; mMetal*=u_materialInfluence; mPaper*=u_materialInfluence; mTV*=u_materialInfluence; mCurtain*=u_materialInfluence;
  reflectivity=max(reflectivity, mGlass*0.86 + mLiquid*0.68 + mMetal*0.76 + mPlastic*0.34 + mWood*0.28 + mTV*0.74); roughness=mix(roughness,0.08,clamp(mGlass+mLiquid,0.0,1.0)); roughness=mix(roughness,0.92,mPaper); roughness=mix(roughness,0.98,mCurtain); glassness=max(glassness, mGlass*0.95 + mLiquid*0.72 + mTV*0.25);
  vec3 color=albedo*u_ambient*ao; vec3 specAccum=vec3(0.0); float bokehSeed=0.0; float shadowDbg=1.0; float selectedShadow=1.0;
  for(int i=0;i<MAX_LIGHTS;i++){ if(i>=u_lightCount) break; if(u_lightExtra[i].y<0.5) continue; float type=u_lightDirType[i].w; vec2 lpos=u_lightPosRad[i].xy; float lz=u_lightPosRad[i].z; float radius=max(u_lightPosRad[i].w,0.001); vec3 L; float att=1.0; if(type<0.5){ vec3 v=vec3(lpos-uv,lz-effD); float dist=length(v); L=v/max(dist,0.0001); att=smoothstep(radius,0.0,dist); att*=att; } else if(type<1.5){ vec3 v=vec3(lpos-uv,lz-effD); float dist=length(v); L=v/max(dist,0.0001); att=smoothstep(radius,0.0,dist); att*=att; vec2 dir=normalize(vec2(u_lightDirType[i].x,u_lightDirType[i].y)); float align=dot(normalize(lpos-uv),dir); float cone=u_lightDirType[i].z; att*=smoothstep(cone,mix(cone,1.0,0.55),align); } else if(type<2.5||type>3.5){ vec2 size=max(u_lightExtra[i].xy,vec2(0.01)); vec3 v=vec3((lpos-uv)/size,(lz-effD)*1.4); float dist=length(v); L=normalize(vec3(lpos-uv,lz-effD)); att=1.0/(1.0+dist*dist*2.3); } else { L=normalize(vec3(-u_lightDirType[i].x,-u_lightDirType[i].y,0.75)); att=1.0; }
    float ndl=max(0.0,dot(n,L)); float sh=shadowRay(uv,effD,i); shadowDbg=min(shadowDbg,sh); if(i==u_selectedLightIndex) selectedShadow=sh; vec3 lc=u_lightColorInt[i].rgb; float intensity=u_lightColorInt[i].a; color += albedo*lc*ndl*att*intensity*sh*u_relightBlend*u_computedLightGain*mask; vec3 V=vec3(0,0,1); vec3 H=normalize(L+V); float gloss=mix(8.0,180.0,1.0-roughness); float fres=pow(1.0-max(0.0,n.z),2.5); float spec=pow(max(dot(n,H),0.0),gloss)*mix(0.12,1.0,reflectivity)*u_specularBoost; spec += fres*reflectivity*(1.0-roughness)*0.24; vec3 specCol=lc*spec*intensity*att*sh*(u_lightExtra[i].w); specAccum += specCol; bokehSeed += luma(specCol)*mix(0.2,1.1,u_lightExtra[i].w); vec2 reflUv=clamp(uv+n.xy*0.018*reflectivity*u_reflectionStrength,0.0,1.0); vec3 refl=texture(u_base,reflUv).rgb; color += refl*reflectivity*(1.0-roughness)*intensity*att*0.08*u_reflectionStrength; }
  vec3 lit=plate+color+specAccum; float emissive=em.r + mTV*0.22 + matIs(mid,11.0)*0.10; vec3 emTint=mix(vec3(0.55,0.72,1.0),vec3(1.0,0.72,0.36),em.b); lit += emTint*emissive; bokehSeed += em.g*emissive + max(0.0,luma(lit)-0.92)*0.45;
  float glassMask=smoothstep(0.42,0.82,glassness)*mask; if(glassMask>0.001){ vec2 refrUv=clamp(uv+n.xy*(0.012+aux.g*0.025)*u_glassRefraction,0.0,1.0); vec3 trans=texture(u_base,refrUv).rgb*u_glassTint; float edge=pow(1.0-max(0.0,n.z),3.0)*u_glassFresnel; vec3 glassCol=mix(lit,trans*u_glassTransmission+specAccum*1.15,glassMask*0.75); glassCol += vec3(edge*u_glassEdge*(0.22+reflectivity*0.9)); lit=mix(lit,glassCol,glassMask); bokehSeed += edge*0.18 + luma(specAccum)*glassMask*0.35; }
  vec3 dbg=lit; if(u_debugMode==1) dbg=baseRaw; else if(u_debugMode==2) dbg=vec3(sampleDepthMap(uv)); else if(u_debugMode==3) dbg=aux.rgb; else if(u_debugMode==4) dbg=vec3(effD); else if(u_debugMode==5) dbg=vec3(reflectivity); else if(u_debugMode==6) dbg=vec3(roughness); else if(u_debugMode==7) dbg=vec3(glassness); else if(u_debugMode==8) dbg=vec3(mask); else if(u_debugMode==9) dbg=n*0.5+0.5; else if(u_debugMode==10) dbg=vec3(u_selectedShadowOnly?selectedShadow:shadowDbg); else if(u_debugMode==11) dbg=vec3(glassMask); else if(u_debugMode==12) dbg=specAccum*1.4; else if(u_debugMode==14) dbg=vec3(fract(mid*0.37),fract(mid*0.61),fract(mid*0.19)); else if(u_debugMode==15) dbg=vec3(emissive,em.g,em.b); else if(u_debugMode==16) dbg=vec3(selectedShadow);
  outColor=vec4(dbg,clamp(bokehSeed,0.0,1.0));
}`;
const postFs = `#version 300 es
precision highp float;
in vec2 v_uv; out vec4 outColor;
uniform sampler2D u_lit; uniform sampler2D u_depth; uniform sampler2D u_aux;
uniform vec2 u_texel; uniform int u_postMode; uniform bool u_applyPost; uniform float u_focusDepth; uniform float u_aperture; uniform float u_nearBlur; uniform float u_farBlur; uniform bool u_dofEnabled; uniform bool u_useBokeh; uniform float u_bokehThreshold; uniform float u_bokehGain; uniform float u_bloomGain; uniform float u_exposure; uniform float u_contrast; uniform float u_saturation; uniform float u_grain; uniform float u_vignette; uniform float u_heightScale; uniform float u_anamorphic; uniform float u_time;
float luma(vec3 c){return dot(c,vec3(0.2126,0.7152,0.0722));}
float effDepth(vec2 uv){vec4 a=texture(u_aux,uv);float d=texture(u_depth,uv).r;return clamp(d-a.g*u_heightScale*a.a,0.0,1.0);}float cocAt(vec2 uv){float d=effDepth(uv);float delta=d-u_focusDepth;float nearA=max(-delta,0.0)*u_nearBlur;float farA=max(delta,0.0)*u_farBlur;return clamp(max(nearA,farA)*(u_aperture*11.0),0.0,7.0);}vec3 toneMap(vec3 c){c*=u_exposure;c=c/(1.0+c);c=mix(vec3(luma(c)),c,u_saturation);c=(c-0.5)*u_contrast+0.5;return clamp(c,0.0,1.0);}void main(){vec2 uv=v_uv;vec4 src4=texture(u_lit,uv);vec3 src=src4.rgb;if(u_postMode==1){outColor=vec4(vec3(cocAt(uv)/7.0),1.0);return;}if(u_postMode==2){float b=src4.a+max(0.0,luma(src)-u_bokehThreshold);outColor=vec4(vec3(b),1.0);return;}if(!u_applyPost){outColor=vec4(clamp(src,0.0,1.0),1.0);return;}vec3 color=src;float radius=cocAt(uv);if(u_dofEnabled){vec2 disk[12];disk[0]=vec2(0);disk[1]=vec2(1,0);disk[2]=vec2(-1,0);disk[3]=vec2(0,1);disk[4]=vec2(0,-1);disk[5]=vec2(.74,.74);disk[6]=vec2(-.74,.74);disk[7]=vec2(.74,-.74);disk[8]=vec2(-.74,-.74);disk[9]=vec2(1.35,0);disk[10]=vec2(-1.35,0);disk[11]=vec2(0,1.35);float myD=effDepth(uv);float mySide=sign(myD-u_focusDepth);vec3 sum=vec3(0);float weight=0.0;vec3 bokehAdd=vec3(0);for(int i=0;i<12;i++){vec2 off=disk[i];off.x*=u_anamorphic;vec2 suv=clamp(uv+off*u_texel*radius*2.1,0.0,1.0);vec4 c4=texture(u_lit,suv);vec3 c=c4.rgb;float sd=effDepth(suv);float ss=sign(sd-u_focusDepth);float compat=(mySide==0.0||ss==mySide||abs(sd-myD)<0.04)?1.0:0.22;float w=(1.10-min(length(disk[i])*0.3,0.85))*compat;sum+=c*w;weight+=w;if(u_useBokeh){float high=max(0.0,c4.a+max(0.0,luma(c)-u_bokehThreshold));bokehAdd+=c*high*0.10*u_bokehGain*smoothstep(0.7,5.0,radius)*compat;}}color=mix(src,sum/max(weight,0.0001),smoothstep(0.2,2.4,radius));color+=bokehAdd;}float bloomSeed=max(0.0,luma(src)-u_bokehThreshold);color+=bloomSeed*u_bloomGain;color=toneMap(color);float grain=fract(sin(dot(gl_FragCoord.xy+vec2(u_time*0.001),vec2(12.9898,78.233)))*43758.5453)-0.5;color+=grain*u_grain;float vig=1.0-distance(uv,vec2(0.5))*u_vignette;color*=clamp(vig,0.0,1.0);outColor=vec4(clamp(color,0.0,1.0),1.0);} `;

function loadImage(src){ return new Promise((resolve,reject)=>{ const img = new Image(); img.onload = ()=>resolve(img); img.onerror = reject; img.src = src; }); }
function setStatus(msg){ el.status.textContent = msg; }
function setDirty(){ STATE.dirty = true; if(!raf) raf = requestAnimationFrame(renderFrame); }
function createCanvasFromImage(img){ const c=document.createElement('canvas'); c.width=W; c.height=H; const cx=c.getContext('2d',{willReadFrequently:true}); cx.drawImage(img,0,0,W,H); return {canvas:c, ctx:cx}; }
function compile(type, src){ const sh=gl.createShader(type); gl.shaderSource(sh,src); gl.compileShader(sh); if(!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(sh)); return sh; }
function link(vs,fs){ const p=gl.createProgram(); gl.attachShader(p,vs); gl.attachShader(p,fs); gl.linkProgram(p); if(!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p)); return p; }
function texFromCanvas(canvas){ const t=gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D,t); gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true); gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,gl.RGBA,gl.UNSIGNED_BYTE,canvas); gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL,false); gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR); gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR); gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE); gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE); return t; }
function updateTextureFromCanvas(tex, canvas){ gl.bindTexture(gl.TEXTURE_2D, tex); gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true); gl.texSubImage2D(gl.TEXTURE_2D,0,0,0,gl.RGBA,gl.UNSIGNED_BYTE,canvas); gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false); }
function initGl(){ gl = el.glCanvas.getContext('webgl2', {antialias:true, premultipliedAlpha:false, preserveDrawingBuffer:true}); if(!gl) throw new Error('WebGL2 not supported'); const vs=compile(gl.VERTEX_SHADER, fullVs), fs1=compile(gl.FRAGMENT_SHADER, litFs), fs2=compile(gl.FRAGMENT_SHADER, postFs); litProgram=link(vs,fs1); postProgram=link(vs,fs2); vao=gl.createVertexArray(); gl.bindVertexArray(vao);
  uniformsLit = {
    u_base: gl.getUniformLocation(litProgram,'u_base'), u_depth: gl.getUniformLocation(litProgram,'u_depth'), u_surface: gl.getUniformLocation(litProgram,'u_surface'), u_aux: gl.getUniformLocation(litProgram,'u_aux'), u_texel: gl.getUniformLocation(litProgram,'u_texel'), u_debugMode: gl.getUniformLocation(litProgram,'u_debugMode'),
    u_ambient: gl.getUniformLocation(litProgram,'u_ambient'), u_platePreserve: gl.getUniformLocation(litProgram,'u_platePreserve'), u_relightBlend: gl.getUniformLocation(litProgram,'u_relightBlend'), u_heightScale: gl.getUniformLocation(litProgram,'u_heightScale'), u_normalStrength: gl.getUniformLocation(litProgram,'u_normalStrength'), u_aoStrength: gl.getUniformLocation(litProgram,'u_aoStrength'), u_reflectionStrength: gl.getUniformLocation(litProgram,'u_reflectionStrength'), u_specularBoost: gl.getUniformLocation(litProgram,'u_specularBoost'), u_glassTransmission: gl.getUniformLocation(litProgram,'u_glassTransmission'), u_glassRefraction: gl.getUniformLocation(litProgram,'u_glassRefraction'), u_glassFresnel: gl.getUniformLocation(litProgram,'u_glassFresnel'), u_glassEdge: gl.getUniformLocation(litProgram,'u_glassEdge'), u_glassTint: gl.getUniformLocation(litProgram,'u_glassTint'), u_shadowStrength: gl.getUniformLocation(litProgram,'u_shadowStrength'), u_shadowBias: gl.getUniformLocation(litProgram,'u_shadowBias'), u_shadowSoftness: gl.getUniformLocation(litProgram,'u_shadowSoftness'), u_shadowSteps: gl.getUniformLocation(litProgram,'u_shadowSteps'), u_liveShadows: gl.getUniformLocation(litProgram,'u_liveShadows'), u_lightCount: gl.getUniformLocation(litProgram,'u_lightCount'), u_lightPosRad: gl.getUniformLocation(litProgram,'u_lightPosRad'), u_lightColorInt: gl.getUniformLocation(litProgram,'u_lightColorInt'), u_lightDirType: gl.getUniformLocation(litProgram,'u_lightDirType'), u_lightExtra: gl.getUniformLocation(litProgram,'u_lightExtra')
  };
  uniformsPost = {
    u_lit: gl.getUniformLocation(postProgram,'u_lit'), u_depth: gl.getUniformLocation(postProgram,'u_depth'), u_aux: gl.getUniformLocation(postProgram,'u_aux'), u_texel: gl.getUniformLocation(postProgram,'u_texel'), u_postMode: gl.getUniformLocation(postProgram,'u_postMode'), u_focusDepth: gl.getUniformLocation(postProgram,'u_focusDepth'), u_aperture: gl.getUniformLocation(postProgram,'u_aperture'), u_nearBlur: gl.getUniformLocation(postProgram,'u_nearBlur'), u_farBlur: gl.getUniformLocation(postProgram,'u_farBlur'), u_dofEnabled: gl.getUniformLocation(postProgram,'u_dofEnabled'), u_useBokeh: gl.getUniformLocation(postProgram,'u_useBokeh'), u_bokehThreshold: gl.getUniformLocation(postProgram,'u_bokehThreshold'), u_bokehGain: gl.getUniformLocation(postProgram,'u_bokehGain'), u_bloomGain: gl.getUniformLocation(postProgram,'u_bloomGain'), u_exposure: gl.getUniformLocation(postProgram,'u_exposure'), u_contrast: gl.getUniformLocation(postProgram,'u_contrast'), u_saturation: gl.getUniformLocation(postProgram,'u_saturation'), u_grain: gl.getUniformLocation(postProgram,'u_grain'), u_vignette: gl.getUniformLocation(postProgram,'u_vignette'), u_heightScale: gl.getUniformLocation(postProgram,'u_heightScale'), u_time: gl.getUniformLocation(postProgram,'u_time')
  };
  
  Object.assign(uniformsLit, {
    u_material: gl.getUniformLocation(litProgram,'u_material'),
    u_emissive: gl.getUniformLocation(litProgram,'u_emissive'),
    u_baseDarkness: gl.getUniformLocation(litProgram,'u_baseDarkness'),
    u_albedoGain: gl.getUniformLocation(litProgram,'u_albedoGain'),
    u_computedLightGain: gl.getUniformLocation(litProgram,'u_computedLightGain'),
    u_shadowLift: gl.getUniformLocation(litProgram,'u_shadowLift'),
    u_highlightSuppress: gl.getUniformLocation(litProgram,'u_highlightSuppress'),
    u_selectedLightIndex: gl.getUniformLocation(litProgram,'u_selectedLightIndex'),
    u_selectedShadowOnly: gl.getUniformLocation(litProgram,'u_selectedShadowOnly'),
    u_materialInfluence: gl.getUniformLocation(litProgram,'u_materialInfluence')
  });
  Object.assign(uniformsPost, {
    u_applyPost: gl.getUniformLocation(postProgram,'u_applyPost'),
    u_anamorphic: gl.getUniformLocation(postProgram,'u_anamorphic')
  });

  textures.base = texFromCanvas(maps.baseCanvas); textures.depth = texFromCanvas(maps.depthCanvas); textures.surface = texFromCanvas(maps.surfaceCanvas); textures.aux = texFromCanvas(maps.auxCanvas); textures.material = texFromCanvas(maps.materialCanvas); textures.emissive = texFromCanvas(maps.emissiveCanvas); recreateBuffers();
}
function recreateBuffers(){
  const q = QUALITY[STATE.quality] || QUALITY.balanced;
  STATE.render.renderScale = q.renderScale;
  STATE.render.shadowSteps = q.shadowSteps;
  renderW = Math.max(320, Math.round(W*q.renderScale));
  renderH = Math.max(180, Math.round(H*q.renderScale));
  if(litTex) gl.deleteTexture(litTex);
  if(depthRb) gl.deleteRenderbuffer(depthRb);
  if(litFbo) gl.deleteFramebuffer(litFbo);

  // RGBA16F render targets require EXT_color_buffer_float and are not reliable on every browser/GPU,
  // especially when the file is opened directly from file://. Use a safe RGBA8 target by default.
  // The mockup still keeps the same pipeline order; this avoids FRAMEBUFFER_INCOMPLETE_ATTACHMENT.
  litTex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, litTex);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, renderW, renderH, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

  depthRb = gl.createRenderbuffer();
  gl.bindRenderbuffer(gl.RENDERBUFFER, depthRb);
  gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, renderW, renderH);

  litFbo = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, litFbo);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, litTex, 0);
  gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depthRb);
  const status = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
  if(status !== gl.FRAMEBUFFER_COMPLETE) {
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    throw new Error('Framebuffer incomplete: 0x' + status.toString(16));
  }
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  setDirty();
}
function lightTypeIndex(type){ return ({point:0,spot:1,area:2,directional:3,emissive:4}[type] ?? 0); }
function viewDebugMode(){ if(STATE.viewMode==='coc' || STATE.viewMode==='bokehSource') return 0; return ({base:1,depth:2,depthAux:3,effectiveDepth:4,surfaceReflect:5,surfaceRough:6,surfaceGlass:7,surfaceMask:8,normal:9,shadow:10,glassMask:11,specular:12,materialID:14,emissive:15,selectedShadow:16}[STATE.viewMode] ?? 0); }
function postMode(){ if(STATE.viewMode==='coc') return 1; if(STATE.viewMode==='bokehSource') return 2; return 0; }
function bindLit(){ gl.useProgram(litProgram); gl.bindVertexArray(vao); gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D,textures.base); gl.uniform1i(uniformsLit.u_base,0); gl.activeTexture(gl.TEXTURE1); gl.bindTexture(gl.TEXTURE_2D,textures.depth); gl.uniform1i(uniformsLit.u_depth,1); gl.activeTexture(gl.TEXTURE2); gl.bindTexture(gl.TEXTURE_2D,textures.surface); gl.uniform1i(uniformsLit.u_surface,2); gl.activeTexture(gl.TEXTURE3); gl.bindTexture(gl.TEXTURE_2D,textures.aux); gl.uniform1i(uniformsLit.u_aux,3); gl.activeTexture(gl.TEXTURE4); gl.bindTexture(gl.TEXTURE_2D,textures.material); gl.uniform1i(uniformsLit.u_material,4); gl.activeTexture(gl.TEXTURE5); gl.bindTexture(gl.TEXTURE_2D,textures.emissive); gl.uniform1i(uniformsLit.u_emissive,5); gl.uniform2f(uniformsLit.u_texel, 1/renderW, 1/renderH); gl.uniform1i(uniformsLit.u_debugMode, viewDebugMode());
  const sr=STATE.render; gl.uniform1f(uniformsLit.u_ambient, sr.ambient); gl.uniform1f(uniformsLit.u_platePreserve, sr.platePreserve); gl.uniform1f(uniformsLit.u_relightBlend, sr.relightBlend); gl.uniform1f(uniformsLit.u_baseDarkness, sr.baseDarkness); gl.uniform1f(uniformsLit.u_albedoGain, sr.albedoGain); gl.uniform1f(uniformsLit.u_computedLightGain, sr.computedLightGain); gl.uniform1f(uniformsLit.u_shadowLift, sr.shadowLift); gl.uniform1f(uniformsLit.u_highlightSuppress, sr.highlightSuppress); gl.uniform1f(uniformsLit.u_heightScale, sr.heightScale); gl.uniform1f(uniformsLit.u_normalStrength, sr.normalStrength); gl.uniform1f(uniformsLit.u_aoStrength, sr.aoStrength); gl.uniform1f(uniformsLit.u_reflectionStrength, sr.reflectionStrength); gl.uniform1f(uniformsLit.u_specularBoost, sr.specularBoost); gl.uniform1f(uniformsLit.u_glassTransmission, sr.glassTransmission); gl.uniform1f(uniformsLit.u_glassRefraction, sr.glassRefraction); gl.uniform1f(uniformsLit.u_glassFresnel, sr.glassFresnel); gl.uniform1f(uniformsLit.u_glassEdge, sr.glassEdge); gl.uniform3fv(uniformsLit.u_glassTint, sr.glassTint); gl.uniform1f(uniformsLit.u_materialInfluence, sr.materialInfluence || 0.0); gl.uniform1f(uniformsLit.u_shadowStrength, sr.shadowStrength); gl.uniform1f(uniformsLit.u_shadowBias, sr.shadowBias); gl.uniform1f(uniformsLit.u_shadowSoftness, sr.shadowSoftness); gl.uniform1i(uniformsLit.u_shadowSteps, sr.shadowSteps|0); gl.uniform1i(uniformsLit.u_liveShadows, sr.liveShadows ? 1 : 0); gl.uniform1i(uniformsLit.u_selectedShadowOnly, sr.selectedShadowOnly ? 1 : 0); gl.uniform1i(uniformsLit.u_selectedLightIndex, Math.max(0, STATE.lights.findIndex(l=>l.id===STATE.selectedLightId)));
  const posrad=new Float32Array(MAX_LIGHTS*4), colint=new Float32Array(MAX_LIGHTS*4), dirtyp=new Float32Array(MAX_LIGHTS*4), extra=new Float32Array(MAX_LIGHTS*4); let c=0; STATE.lights.slice(0,MAX_LIGHTS).forEach(l=>{ if(!l) return; const i=c*4; posrad[i]=l.x; posrad[i+1]=l.y; posrad[i+2]=l.z; posrad[i+3]=l.radius || 0.25; colint[i]=l.color[0]; colint[i+1]=l.color[1]; colint[i+2]=l.color[2]; colint[i+3]=l.intensity; const ang=Math.max(-1,Math.min(1,l.cone ?? 0.82)); dirtyp[i]=l.dirX||0; dirtyp[i+1]=l.dirY||1; dirtyp[i+2]=ang; dirtyp[i+3]=lightTypeIndex(l.type); extra[i]=l.shadow ?? 0.5; extra[i+1]=l.enabled ? 1 : 0; extra[i+2]=(l.shadow??0)>0.001 && l.type!=='emissive' ? 1 : 0; extra[i+3]=l.spec ?? 1; if(l.type==='area' || l.type==='emissive'){ extra[i]=l.width || 0.18; extra[i+1]=l.height || 0.18; extra[i+2]=(l.shadow??0)>0.001 && l.type!=='emissive' ? 1 : 0; extra[i+3]=l.spec ?? 1; } c++; }); gl.uniform1i(uniformsLit.u_lightCount, c); gl.uniform4fv(uniformsLit.u_lightPosRad, posrad); gl.uniform4fv(uniformsLit.u_lightColorInt, colint); gl.uniform4fv(uniformsLit.u_lightDirType, dirtyp); gl.uniform4fv(uniformsLit.u_lightExtra, extra); }
function bindPost(time){ gl.useProgram(postProgram); gl.bindVertexArray(vao); gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D,litTex); gl.uniform1i(uniformsPost.u_lit,0); gl.activeTexture(gl.TEXTURE1); gl.bindTexture(gl.TEXTURE_2D,textures.depth); gl.uniform1i(uniformsPost.u_depth,1); gl.activeTexture(gl.TEXTURE2); gl.bindTexture(gl.TEXTURE_2D,textures.aux); gl.uniform1i(uniformsPost.u_aux,2); gl.uniform2f(uniformsPost.u_texel,1/renderW,1/renderH); gl.uniform1i(uniformsPost.u_postMode, postMode()); gl.uniform1i(uniformsPost.u_applyPost, STATE.viewMode==='final' ? 1 : 0); const sr=STATE.render; gl.uniform1f(uniformsPost.u_focusDepth, sr.focusDepth); gl.uniform1f(uniformsPost.u_aperture, sr.aperture); gl.uniform1f(uniformsPost.u_nearBlur, sr.nearBlur); gl.uniform1f(uniformsPost.u_farBlur, sr.farBlur); gl.uniform1i(uniformsPost.u_dofEnabled, (STATE.viewMode==='final' && sr.dofEnabled) ? 1 : 0); gl.uniform1i(uniformsPost.u_useBokeh, sr.useBokeh ? 1 : 0); gl.uniform1f(uniformsPost.u_bokehThreshold, sr.bokehThreshold); gl.uniform1f(uniformsPost.u_bokehGain, sr.bokehGain); gl.uniform1f(uniformsPost.u_bloomGain, sr.bloomGain); gl.uniform1f(uniformsPost.u_exposure, sr.exposure); gl.uniform1f(uniformsPost.u_contrast, sr.contrast); gl.uniform1f(uniformsPost.u_saturation, sr.saturation); gl.uniform1f(uniformsPost.u_grain, sr.grain); gl.uniform1f(uniformsPost.u_vignette, sr.vignette); gl.uniform1f(uniformsPost.u_heightScale, sr.heightScale); gl.uniform1f(uniformsPost.u_anamorphic, sr.anamorphic); gl.uniform1f(uniformsPost.u_time, time); }
function renderFrame(ts){ raf = 0; if(lastTs){ const dt = ts-lastTs; fps = fps*0.84 + (1000/Math.max(dt,1))*0.16; } lastTs = ts; if(!STATE.dirty || !gl) return; STATE.dirty = false; gl.disable(gl.BLEND); gl.bindFramebuffer(gl.FRAMEBUFFER, litFbo); gl.viewport(0,0,renderW,renderH); bindLit(); gl.drawArrays(gl.TRIANGLES,0,3); gl.bindFramebuffer(gl.FRAMEBUFFER, null); gl.viewport(0,0,el.glCanvas.width,el.glCanvas.height); bindPost(ts||performance.now()); gl.drawArrays(gl.TRIANGLES,0,3); drawOverlay(); updateStatus(); }
function updateStatus(){ setStatus(`view ${STATE.viewMode} • ${renderW}×${renderH} • ${STATE.quality} • ${fps?fps.toFixed(1):'—'} fps • lights ${STATE.lights.length}`); }
function getSelectedLight(){ return STATE.lights.find(l=>l.id===STATE.selectedLightId) || null; }
function buildToolbar(){ const groups = [
  {title:'View', items:[['moveLight','💡','Light'],['focus','🎯','Focus'],['probe','🧪','Probe'],['pan','🖐️','Pan']]},
  {title:'Surface', items:[['surfReflect','✨','Reflect'],['surfRough','🪵','Rough'],['surfGlass','🧊','Glass'],['surfMask','🎭','Mask']]},
  {title:'DepthAux', items:[['auxDepth','⬛','Depth'],['auxHeight','🧱','Height'],['auxOcc','🌑','Occluder'],['auxMask','✅','Valid']]}
];
  el.toolbar.innerHTML=''; groups.forEach(g=>{ const title=document.createElement('div'); title.className='group-title'; title.textContent=g.title; el.toolbar.appendChild(title); g.items.forEach(([id,icon,label])=>{ const b=document.createElement('button'); b.className='tool-btn'+(STATE.tool===id?' active':''); b.innerHTML=`<div class="icon">${icon}</div><div class="label">${label}</div>`; b.onclick=()=>{ STATE.tool=id; updateHelp(); buildToolbar(); }; el.toolbar.appendChild(b); }); }); }
function section(title, body, open=true){ return `<details class="panel-section" ${open?'open':''}><summary><span>${title}</span><span class="small">▾</span></summary><div class="section-body">${body}</div></details>`; }
function slider(id,label,min,max,step,value){ return `<div class="row compact"><label for="${id}">${label}</label><input id="${id}" type="range" min="${min}" max="${max}" step="${step}" value="${value}"><div class="val" id="${id}_val"></div></div>`; }
function buildSidebar(){ const sel = getSelectedLight(); el.sidebar.innerHTML = [
  section('Scene presets', `
    <div class="notice">v6: hardcoded real maps + MaterialID + EmissiveAux. Wet workflow usunięty; bokeh działa jako końcowy camera pass. Dodane hotkeys i direction gizmo.</div>
    <div class="btn-row">
      <button data-preset="noirDefault">Noir default</button>
      <button data-preset="deskLampFocus">Desk lamp</button>
      <button data-preset="coldWindow">Cold window</button>
      <button data-preset="tvMood">TV mood</button>
      <button data-preset="previewSoft">Preview soft</button>
    </div>
  `),
  section('View / debug', `
    <div class="row"><label>View mode</label><select id="viewMode">${VIEW_OPTIONS.map(v=>`<option value="${v}" ${STATE.viewMode===v?'selected':''}>${v}</option>`).join('')}</select></div>
    <div class="chips"><span class="chip">Base</span><span class="chip">Depth</span><span class="chip">DepthAux</span><span class="chip">Surface</span><span class="chip">Normal</span><span class="chip">Shadow</span><span class="chip">Bokeh</span><span class="chip">COC</span></div>
  `),
  section('Performance', `
    <div class="row"><label>Quality</label><select id="qualitySel">${Object.keys(QUALITY).map(k=>`<option value="${k}" ${STATE.quality===k?'selected':''}>${k}</option>`).join('')}</select></div>
    <div class="row"><label><input type="checkbox" id="showHandles" ${STATE.render.showHandles?'checked':''}> Show handles</label><span class="hint">overlay</span></div>
    <div class="row"><label><input type="checkbox" id="selectedShadowOnly" ${STATE.render.selectedShadowOnly?'checked':''}> Selected shadow debug</label><span class="hint">debug</span></div>
    <div class="row"><label><input type="checkbox" id="liveShadows" ${STATE.render.liveShadows?'checked':''}> Live shadows</label><span class="hint">depth ray</span></div>
  `),
  section('Global lighting', `
    ${slider('ambient','Ambient',0,0.4,0.005,STATE.render.ambient)}
    ${slider('baseDarkness','Base darkness',0,1,0.01,STATE.render.baseDarkness)}
    ${slider('albedoGain','Albedo gain',0,2,0.01,STATE.render.albedoGain)}
    ${slider('computedLightGain','Computed light',0,2.5,0.01,STATE.render.computedLightGain)}
    ${slider('shadowLift','Shadow lift',0,1,0.01,STATE.render.shadowLift)}
    ${slider('highlightSuppress','Highlight suppress',0,1,0.01,STATE.render.highlightSuppress)}
    ${slider('platePreserve','Plate preserve',0,1,0.01,STATE.render.platePreserve)}
    ${slider('relightBlend','Relight blend',0,1,0.01,STATE.render.relightBlend)}
    ${slider('heightScale','Height influence',0,0.5,0.005,STATE.render.heightScale)}
    ${slider('normalStrength','Normal strength',0.5,4,0.01,STATE.render.normalStrength)}
    ${slider('aoStrength','Contact AO',0,1,0.01,STATE.render.aoStrength)}
    ${slider('reflectionStrength','Reflections',0,1,0.01,STATE.render.reflectionStrength)}
    ${slider('specularBoost','Spec boost',0,3,0.01,STATE.render.specularBoost)}
  `),
  section('Glass / material response', `
    ${slider('glassTransmission','Transmission',0,1.4,0.01,STATE.render.glassTransmission)}
    ${slider('glassRefraction','Refraction',0,1.5,0.01,STATE.render.glassRefraction)}
    ${slider('glassFresnel','Fresnel',0,2,0.01,STATE.render.glassFresnel)}
    ${slider('glassEdge','Edge highlight',0,2,0.01,STATE.render.glassEdge)}
    <div class="row"><label>Glass tint</label><input id="glassTint" type="color" value="${hex(STATE.render.glassTint)}"></div>
    ${slider('materialInfluence','Material ID influence',0,1,0.01,STATE.render.materialInfluence)}
    <div class="hint">MaterialID is loaded as a real map, but final-render influence is off by default to prevent visible segmentation/primitive leakage. Surface R=reflectivity, G=roughness, B=glass/transmission, A=material mask.</div>
  `),
  section('Shadows', `
    ${slider('shadowStrength','Strength',0,1,0.01,STATE.render.shadowStrength)}
    ${slider('shadowBias','Bias',0,0.1,0.001,STATE.render.shadowBias)}
    ${slider('shadowSoftness','Softness',0,1,0.01,STATE.render.shadowSoftness)}
    <div class="row"><label>Shadow steps</label><div class="val">${STATE.render.shadowSteps}</div></div>
  `),
  section('Camera / post', `
    <div class="row"><label><input type="checkbox" id="dofEnabled" ${STATE.render.dofEnabled?'checked':''}> Enable DOF</label><span class="hint">camera pass</span></div>
    <div class="row"><label><input type="checkbox" id="useBokeh" ${STATE.render.useBokeh?'checked':''}> Enable bokeh</label><span class="hint">highlight pass</span></div>
    ${slider('focusDepth','Focus depth',0,1,0.005,STATE.render.focusDepth)}
    ${slider('aperture','Aperture',0,0.35,0.005,STATE.render.aperture)}
    ${slider('nearBlur','Near blur',0,4,0.05,STATE.render.nearBlur)}
    ${slider('farBlur','Far blur',0,4,0.05,STATE.render.farBlur)}
    ${slider('bokehThreshold','Bokeh threshold',0.5,1.2,0.01,STATE.render.bokehThreshold)}
    ${slider('bokehGain','Bokeh gain',0,2.4,0.01,STATE.render.bokehGain)}
    ${slider('bloomGain','Bloom',0,1,0.01,STATE.render.bloomGain)}
    ${slider('exposure','Exposure',0.4,2.0,0.01,STATE.render.exposure)}
    ${slider('contrast','Contrast',0.6,1.6,0.01,STATE.render.contrast)}
    ${slider('saturation','Saturation',0,1.6,0.01,STATE.render.saturation)}
    ${slider('grain','Grain',0,0.08,0.001,STATE.render.grain)}
    ${slider('vignette','Vignette',0,0.5,0.01,STATE.render.vignette)}
    ${slider('anamorphic','Anamorphic',0.4,2.4,0.01,STATE.render.anamorphic)}
  `, true),
  section('Material ID / Object ID', `
    <div class="row"><label>Paint material</label><select id="brushMaterial">${[0,1,2,3,4,5,6,7,8,9,10,11,12,13].map(i=>`<option value="${i}" ${STATE.brush.materialId===i?'selected':''}>${i} - ${['Wall','Wood','Glass tumbler','Bottle glass','Liquid','Ashtray glass','Phone plastic','Lamp metal','Paper/cardboard','TV screen','Curtain','Window glass','Radiator metal','Cabinet/TV body'][i]}</option>`).join('')}</select></div>
    <div class="hint">MaterialID pozwala shaderowi odróżnić szkło, drewno, papier, plastik, metal, ekran i okno. To jest kluczowe dla realizmu.</div>
  `, false),
  section('Paint tools', `
    <div class="row"><label>Brush preset</label><select id="brushPreset">${Object.keys(BRUSH_PRESETS).map(k=>`<option value="${k}" ${STATE.brush.preset===k?'selected':''}>${BRUSH_PRESETS[k].label}</option>`).join('')}</select></div>
    ${slider('brushSize','Brush size',1,140,1,STATE.brush.size)}
    ${slider('brushHard','Hardness',0,1,0.01,STATE.brush.hardness)}
    ${slider('brushStrength','Strength',0,1,0.01,STATE.brush.strength)}
    <div class="hint">Kliknij narzędzie po lewej, potem maluj bezpośrednio po scenie. Surface = materiały, DepthAux = depth/height/occluder/mask.</div>
  `, false),
  section('Lights', `
    <div class="btn-row">
      <button id="addPoint">+ Point</button>
      <button id="addSpot">+ Spot</button>
      <button id="addArea">+ Area</button>
      <button id="addDirectional">+ Dir</button>
      <button id="addEmissive">+ Emissive</button>
    </div>
    <div class="light-list" id="lightList"></div>
  `),
  section('Selected light', sel ? `
    <div class="row"><label>Name</label><input id="lightName" type="text" value="${sel.name}"></div>
    <div class="row"><label>Type</label><select id="lightType">${['point','spot','area','directional','emissive'].map(t=>`<option value="${t}" ${sel.type===t?'selected':''}>${t}</option>`).join('')}</select></div>
    <div class="grid-3"><div><label>X</label><input id="lightX" type="number" step="0.001" min="0" max="1" value="${sel.x.toFixed(3)}"></div><div><label>Y</label><input id="lightY" type="number" step="0.001" min="0" max="1" value="${sel.y.toFixed(3)}"></div><div><label>Z</label><input id="lightZ" type="number" step="0.001" min="0" max="1" value="${sel.z.toFixed(3)}"></div></div>
    ${slider('lightRadius','Radius',0.02,1.2,0.005,sel.radius || 0.25)}
    ${slider('lightIntensity','Intensity',0,5,0.01,sel.intensity)}
    ${slider('lightShadow','Shadow',0,1,0.01,sel.shadow||0)}
    ${slider('lightSpec','Specular',0,3,0.01,sel.spec||1)}
    ${slider('lightBokeh','Bokeh',0,3,0.01,sel.bokeh||1)}
    <div class="row"><label>Color</label><input id="lightColor" type="color" value="${hex(sel.color)}"></div>
    ${(sel.type==='spot' || sel.type==='directional') ? `<div class="grid-2"><div><label>Dir X</label><input id="lightDirX" type="number" step="0.001" min="-1" max="1" value="${(sel.dirX||0).toFixed(3)}"></div><div><label>Dir Y</label><input id="lightDirY" type="number" step="0.001" min="-1" max="1" value="${(sel.dirY||0).toFixed(3)}"></div></div>` : ''}
    ${sel.type==='spot' ? slider('lightCone','Cone cos',0.1,0.99,0.005,sel.cone||0.82) : ''}
    ${(sel.type==='area' || sel.type==='emissive') ? `<div class="grid-2"><div><label>Width</label><input id="lightWidth" type="number" step="0.001" min="0.01" max="1" value="${(sel.width||0.2).toFixed(3)}"></div><div><label>Height</label><input id="lightHeight" type="number" step="0.001" min="0.01" max="1" value="${(sel.height||0.2).toFixed(3)}"></div></div>` : ''}
    <div class="btn-row"><button id="duplicateLight">Duplicate</button><button class="danger" id="deleteLight">Delete</button></div>
  ` : `<div class="hint">Brak zaznaczonego światła.</div>`),
  section('Hotkeys', `
    <div class="hint">L light move • R rotate direction • F focus • P probe • H pan • Delete remove light • Ctrl+D duplicate • S solo/restore • 1-5 presets • [ / ] debug views • G handles • 0 final view</div>
  `, false),
  section('Probe / export', `
    <div class="probe" id="probeOutput">${formatProbe()}</div>
    <div class="btn-row"><button id="exportPng">Export PNG</button><button id="exportSurface">Export surface</button><button id="exportAux">Export depthAux</button><button id="exportMaterial">Export materialID</button><button id="exportEmissive">Export emissive</button><button id="exportJson">Export scene JSON</button></div>
  `, false)
  ].join('');
  bindSidebar();
  renderLightList();
}
function bindSlider(id, getterSetter, decimals=2, rerender=true){ const input=$('#'+id), val=$('#'+id+'_val'); if(!input) return; const update=()=>{ const v=parseFloat(input.value); getterSetter(v); if(val) val.textContent=v.toFixed(decimals); if(rerender) setDirty(); }; input.addEventListener('input', update); update(); }
function bindSidebar(){
  document.querySelectorAll('[data-preset]').forEach(b=>b.onclick=()=>{ PRESETS[b.dataset.preset](); buildSidebar(); buildToolbar(); setDirty(); updateHelp(); });
  const vm=$('#viewMode'); if(vm) vm.onchange=()=>{ STATE.viewMode=vm.value; setDirty(); };
  const q=$('#qualitySel'); if(q) q.onchange=()=>{ STATE.quality=q.value; recreateBuffers(); buildSidebar(); };
  const showHandles=$('#showHandles'); if(showHandles) showHandles.onchange=()=>{ STATE.render.showHandles=showHandles.checked; drawOverlay(); };
  const liveShadows=$('#liveShadows'); if(liveShadows) liveShadows.onchange=()=>{ STATE.render.liveShadows=liveShadows.checked; setDirty(); }; const selectedShadowOnly=$('#selectedShadowOnly'); if(selectedShadowOnly) selectedShadowOnly.onchange=()=>{ STATE.render.selectedShadowOnly=selectedShadowOnly.checked; setDirty(); };
  [['ambient','ambient'],['baseDarkness','baseDarkness'],['albedoGain','albedoGain'],['computedLightGain','computedLightGain'],['shadowLift','shadowLift'],['highlightSuppress','highlightSuppress'],['platePreserve','platePreserve'],['relightBlend','relightBlend'],['heightScale','heightScale'],['normalStrength','normalStrength'],['aoStrength','aoStrength'],['reflectionStrength','reflectionStrength'],['specularBoost','specularBoost'],['glassTransmission','glassTransmission'],['glassRefraction','glassRefraction'],['glassFresnel','glassFresnel'],['glassEdge','glassEdge'],['materialInfluence','materialInfluence'],['shadowStrength','shadowStrength'],['shadowBias','shadowBias'],['shadowSoftness','shadowSoftness'],['focusDepth','focusDepth'],['aperture','aperture'],['nearBlur','nearBlur'],['farBlur','farBlur'],['bokehThreshold','bokehThreshold'],['bokehGain','bokehGain'],['bloomGain','bloomGain'],['exposure','exposure'],['contrast','contrast'],['saturation','saturation'],['grain','grain'],['vignette','vignette'],['anamorphic','anamorphic']].forEach(([id,key])=>bindSlider(id,v=>STATE.render[key]=v, id==='heightScale'||id==='shadowBias'||id==='grain'?3:2));
  const dof=$('#dofEnabled'); if(dof) dof.onchange=()=>{ STATE.render.dofEnabled=dof.checked; setDirty(); };
  const bok=$('#useBokeh'); if(bok) bok.onchange=()=>{ STATE.render.useBokeh=bok.checked; setDirty(); };
  const tint=$('#glassTint'); if(tint) tint.onchange=()=>{ STATE.render.glassTint=hexRgb(tint.value); setDirty(); };
  const bp=$('#brushPreset'); if(bp) bp.onchange=()=>STATE.brush.preset=bp.value; const bm=$('#brushMaterial'); if(bm) bm.onchange=()=>STATE.brush.materialId=parseInt(bm.value,10)||0; bindSlider('brushSize',v=>STATE.brush.size=v,0,false); bindSlider('brushHard',v=>STATE.brush.hardness=v,2,false); bindSlider('brushStrength',v=>STATE.brush.strength=v,2,false);
  const add = (id,type)=>{ const b=$(id); if(b) b.onclick=()=>addLight(type); }; add('#addPoint','point'); add('#addSpot','spot'); add('#addArea','area'); add('#addDirectional','directional'); add('#addEmissive','emissive');
  bindSelectedLightControls();
  const e1=$('#exportPng'); if(e1) e1.onclick=()=>downloadUrl(el.glCanvas.toDataURL('image/png'),'amigo_v5_render.png');
  const e2=$('#exportSurface'); if(e2) e2.onclick=()=>downloadUrl(maps.surfaceCanvas.toDataURL('image/png'),'amigo_v5_surface.png');
  const e3=$('#exportAux'); if(e3) e3.onclick=()=>downloadUrl(maps.auxCanvas.toDataURL('image/png'),'amigo_v5_depthaux.png');
  const emat=$('#exportMaterial'); if(emat) emat.onclick=()=>downloadUrl(maps.materialCanvas.toDataURL('image/png'),'amigo_v6_material_id.png'); const eem=$('#exportEmissive'); if(eem) eem.onclick=()=>downloadUrl(maps.emissiveCanvas.toDataURL('image/png'),'amigo_v6_emissive_aux.png'); const e4=$('#exportJson'); if(e4) e4.onclick=()=>downloadBlob(new Blob([JSON.stringify({render:STATE.render,lights:STATE.lights},null,2)],{type:'application/json'}),'amigo_v5_scene.json');
}
function renderLightList(){ const root=$('#lightList'); if(!root) return; root.innerHTML=''; STATE.lights.forEach(l=>{ const d=document.createElement('div'); d.className='light-item'+(l.id===STATE.selectedLightId?' selected':''); d.innerHTML=`<div class="light-top"><div><div><strong>${l.name}</strong></div><div class="small">${l.type}</div></div><div class="swatch" style="background:${hex(l.color)}"></div></div><div class="btn-row"><button data-act="select">Select</button><button data-act="toggle">${l.enabled?'On':'Off'}</button><button data-act="solo">Solo</button><button class="danger" data-act="delete">Delete</button></div>`; d.querySelector('[data-act="select"]').onclick=()=>{ STATE.selectedLightId=l.id; buildSidebar(); drawOverlay(); };
    d.querySelector('[data-act="toggle"]').onclick=()=>{ l.enabled=!l.enabled; buildSidebar(); setDirty(); };
    d.querySelector('[data-act="solo"]').onclick=()=>{ STATE.lights.forEach(x=>x.enabled=(x.id===l.id)); STATE.selectedLightId=l.id; buildSidebar(); setDirty(); };
    d.querySelector('[data-act="delete"]').onclick=()=>{ STATE.selectedLightId=l.id; deleteSelectedLight(); };
    root.appendChild(d);
  }); }
function bindSelectedLightControls(){ const l=getSelectedLight(); if(!l) return; const bind=(id,fn,evt='change')=>{ const n=$('#'+id); if(!n) return; n.addEventListener(evt,()=>{ fn(n); setDirty(); drawOverlay(); buildSidebar(); }); };
  bind('lightName',n=>l.name=n.value); bind('lightType',n=>l.type=n.value); bind('lightX',n=>l.x=clamp(parseFloat(n.value)||0,0,1)); bind('lightY',n=>l.y=clamp(parseFloat(n.value)||0,0,1)); bind('lightZ',n=>l.z=clamp(parseFloat(n.value)||0,0,1)); bindSlider('lightRadius',v=>l.radius=v,2); bindSlider('lightIntensity',v=>l.intensity=v,2); bindSlider('lightShadow',v=>l.shadow=v,2); bindSlider('lightSpec',v=>l.spec=v,2); bindSlider('lightBokeh',v=>l.bokeh=v,2); bind('lightColor',n=>l.color=hexRgb(n.value)); bind('lightDirX',n=>l.dirX=clamp(parseFloat(n.value)||0,-1,1)); bind('lightDirY',n=>l.dirY=clamp(parseFloat(n.value)||0,-1,1)); bind('lightCone',n=>l.cone=clamp(parseFloat(n.value)||0,0.1,0.99),'input'); bind('lightWidth',n=>l.width=Math.max(0.01,parseFloat(n.value)||0.2)); bind('lightHeight',n=>l.height=Math.max(0.01,parseFloat(n.value)||0.2)); bindSlider('lightAngle',v=>{ const rad=v*Math.PI/180; l.dirX=Math.cos(rad); l.dirY=Math.sin(rad); },0); const del=$('#deleteLight'); if(del) del.onclick=()=>deleteSelectedLight(); const dup=$('#duplicateLight'); if(dup) dup.onclick=()=>duplicateSelectedLight(); }
function addLight(type){ const preset={ point:{name:'Point',x:0.58,y:0.52,z:0.20,radius:0.22,intensity:1.0,color:[1,0.82,0.58],dirX:0,dirY:1,cone:0.82,shadow:0.6,spec:1.0,bokeh:0.8,enabled:true,width:0.14,height:0.14}, spot:{name:'Spot',x:0.72,y:0.45,z:0.26,radius:0.48,intensity:1.8,color:[1,0.72,0.4],dirX:-0.4,dirY:0.5,cone:0.82,shadow:0.82,spec:1.2,bokeh:1.1,enabled:true,width:0.2,height:0.2}, area:{name:'Area',x:0.58,y:0.28,z:0.90,radius:0.35,intensity:1.1,color:[0.60,0.70,1.0],dirX:0,dirY:1,cone:1,shadow:0.42,spec:0.7,bokeh:1.0,enabled:true,width:0.22,height:0.28}, directional:{name:'Directional',x:0.5,y:0.2,z:0.95,radius:0.55,intensity:0.65,color:[0.58,0.68,1.0],dirX:-0.5,dirY:0.7,cone:1,shadow:0.38,spec:0.5,bokeh:0.4,enabled:true,width:0.2,height:0.2}, emissive:{name:'Emissive',x:0.22,y:0.40,z:0.56,radius:0.24,intensity:0.7,color:[0.38,0.76,1.0],dirX:0,dirY:1,cone:1,shadow:0.0,spec:0.5,bokeh:0.7,enabled:true,width:0.16,height:0.1} }[type]; const l=Object.assign({id:uid(),type},preset); l.name += ' ' + (STATE.lights.filter(x=>x.type===type).length+1); STATE.lights.push(l); STATE.selectedLightId=l.id; buildSidebar(); setDirty(); }
function deleteSelectedLight(){ const idx=STATE.lights.findIndex(l=>l.id===STATE.selectedLightId); if(idx<0) return; STATE.lights.splice(idx,1); STATE.selectedLightId = STATE.lights[Math.min(idx,STATE.lights.length-1)]?.id || null; buildSidebar(); setDirty(); drawOverlay(); }
function duplicateSelectedLight(){ const l=getSelectedLight(); if(!l) return; const c=JSON.parse(JSON.stringify(l)); c.id=uid(); c.name += ' Copy'; c.x=clamp(c.x+0.03,0,1); c.y=clamp(c.y+0.03,0,1); STATE.lights.push(c); STATE.selectedLightId=c.id; buildSidebar(); setDirty(); }
function updateHelp(){ el.help.innerHTML = TOOL_INFO[STATE.tool] || ''; }
function drawOverlay(){ ctx2d.clearRect(0,0,W,H); if(STATE.render.showHandles){ STATE.lights.forEach(l=>{ if(!l.enabled) return; const x=l.x*W, y=l.y*H; ctx2d.save(); ctx2d.strokeStyle = l.id===STATE.selectedLightId ? '#6bd8ff' : 'rgba(255,255,255,.65)'; ctx2d.fillStyle = l.id===STATE.selectedLightId ? 'rgba(107,216,255,.15)' : 'rgba(255,255,255,.06)'; ctx2d.lineWidth = 2; if(l.type==='area' || l.type==='emissive'){ const w=(l.width||0.2)*W, h=(l.height||0.2)*H; ctx2d.beginPath(); ctx2d.rect(x-w/2,y-h/2,w,h); ctx2d.fill(); ctx2d.stroke(); } else { ctx2d.beginPath(); ctx2d.arc(x,y,10,0,Math.PI*2); ctx2d.fill(); ctx2d.stroke(); const rr=(l.radius||0.22)*W; ctx2d.globalAlpha=0.16; ctx2d.beginPath(); ctx2d.arc(x,y,rr,0,Math.PI*2); ctx2d.stroke(); ctx2d.globalAlpha=1; }
      if(l.type==='spot' || l.type==='directional'){ ctx2d.beginPath(); ctx2d.moveTo(x,y); ctx2d.lineTo(x+(l.dirX||0)*120,y+(l.dirY||0)*120); ctx2d.stroke(); }
      ctx2d.restore();
    }); }
  if(STATE.probe){ const x=STATE.probe.u*W, y=STATE.probe.v*H; ctx2d.save(); ctx2d.strokeStyle='#ffd166'; ctx2d.lineWidth=2; ctx2d.beginPath(); ctx2d.arc(x,y,8,0,Math.PI*2); ctx2d.moveTo(x-12,y); ctx2d.lineTo(x+12,y); ctx2d.moveTo(x,y-12); ctx2d.lineTo(x,y+12); ctx2d.stroke(); ctx2d.restore(); }
  if(STATE.drag && STATE.drag.type==='paint' && STATE.drag.last){ const p=STATE.drag.last; ctx2d.save(); ctx2d.strokeStyle='rgba(255,255,255,.92)'; ctx2d.lineWidth=2; ctx2d.beginPath(); ctx2d.arc(p.u*W,p.v*H,STATE.brush.size,0,Math.PI*2); ctx2d.stroke(); ctx2d.restore(); }
}
function canvasUv(evt){ const r=el.glCanvas.getBoundingClientRect(); return {u:clamp((evt.clientX-r.left)/r.width,0,1), v:clamp((evt.clientY-r.top)/r.height,0,1)}; }
function pickLight(u,v){ let best=null,bestD=1e9; for(const l of STATE.lights){ if(l.type==='area' || l.type==='emissive'){ const d=Math.max(Math.abs(u-l.x)/(l.width||0.2),Math.abs(v-l.y)/(l.height||0.2)); if(d<0.7 && d<bestD){best=l;bestD=d;} } else { const d=Math.hypot(u-l.x,v-l.y); if(d<0.05 && d<bestD){best=l;bestD=d;} } } return best; }
function readPx(ctx,u,v){ const x=clamp(Math.round(u*(W-1)),0,W-1), y=clamp(Math.round(v*(H-1)),0,H-1); const d=ctx.getImageData(x,y,1,1).data; return [d[0]/255,d[1]/255,d[2]/255,d[3]/255]; }
function updateProbe(u,v){ const depth=readPx(maps.depthCtx,u,v)[0], surf=readPx(maps.surfaceCtx,u,v), aux=readPx(maps.auxCtx,u,v), mat=readPx(maps.materialCtx,u,v), em=readPx(maps.emissiveCtx,u,v); const eff=clamp(depth - aux[1]*STATE.render.heightScale*aux[3],0,1); STATE.probe={u,v,depth,surf,aux,mat,em,eff}; const p=$('#probeOutput'); if(p) p.textContent=formatProbe(); drawOverlay(); }
function formatProbe(){ if(!STATE.probe) return 'Kliknij w scenie w trybie Probe.'; const p=STATE.probe; return [`uv              ${p.u.toFixed(3)}, ${p.v.toFixed(3)}`,`depth            ${p.depth.toFixed(3)}`,`aux.height       ${p.aux[1].toFixed(3)}`,`aux.occluder     ${p.aux[2].toFixed(3)}`,`aux.mask         ${p.aux[3].toFixed(3)}`,`effectiveDepth   ${p.eff.toFixed(3)}`,`reflectivity     ${p.surf[0].toFixed(3)}`,`roughness        ${p.surf[1].toFixed(3)}`,`glass            ${p.surf[2].toFixed(3)}`,`surfaceMask      ${p.surf[3].toFixed(3)}`,`materialID       ${Math.round(p.mat[0]*15)}`,`emissive         ${p.em[0].toFixed(3)}`,`bokehEligible    ${p.em[1].toFixed(3)}`].join('\n'); }
function paintAt(u,v){ const radius=STATE.brush.size, hard=STATE.brush.hardness, str=STATE.brush.strength, preset=BRUSH_PRESETS[STATE.brush.preset]; let ctx=null, tex=null, channel=0, targetVal=0; if(STATE.tool==='paintMaterial'){ ctx=maps.materialCtx; tex=textures.material; channel=0; targetVal=(STATE.brush.materialId||0)/15; } else if(STATE.tool.startsWith('surf')){ ctx=maps.surfaceCtx; tex=textures.surface; if(STATE.tool==='surfReflect'){channel=0; targetVal=preset.reflect;} if(STATE.tool==='surfRough'){channel=1; targetVal=preset.rough;} if(STATE.tool==='surfGlass'){channel=2; targetVal=preset.glass;} if(STATE.tool==='surfMask'){channel=3; targetVal=preset.mask;} } else { ctx=maps.auxCtx; tex=textures.aux; if(STATE.tool==='auxDepth'){channel=0; targetVal=preset.depth;} if(STATE.tool==='auxHeight'){channel=1; targetVal=preset.height;} if(STATE.tool==='auxOcc'){channel=2; targetVal=preset.occ;} if(STATE.tool==='auxMask'){channel=3; targetVal=preset.auxMask;} }
  if(!ctx) return; const img=ctx.getImageData(0,0,W,H), data=img.data, cx=u*W, cy=v*H; const x0=Math.max(0,Math.floor(cx-radius-1)), x1=Math.min(W-1,Math.ceil(cx+radius+1)), y0=Math.max(0,Math.floor(cy-radius-1)), y1=Math.min(H-1,Math.ceil(cy+radius+1));
  for(let y=y0;y<=y1;y++) for(let x=x0;x<=x1;x++){ const dx=(x-cx)/radius, dy=(y-cy)/radius, d=Math.sqrt(dx*dx+dy*dy); if(d>1) continue; const fall=Math.pow(1-d, lerp(3.2,0.7,hard))*str; const idx=(y*W+x)*4+channel; const src=data[idx]/255, nv=src*(1-fall)+targetVal*fall; data[idx]=Math.round(clamp(nv,0,1)*255); }
  ctx.putImageData(img,0,0); updateTextureFromCanvas(tex, ctx.canvas); setDirty(); }
function pointerDown(evt){ const uv=canvasUv(evt); if(STATE.tool==='rotateLight'){ const l=getSelectedLight(); if(l){ STATE.drag={type:'direction'}; const rad=Math.atan2(uv.v-l.y, uv.u-l.x); l.dirX=Math.cos(rad); l.dirY=Math.sin(rad); buildSidebar(); setDirty(); drawOverlay(); } }
    else if(STATE.tool==='moveLight'){ const hit=pickLight(uv.u,uv.v) || getSelectedLight(); if(hit){ STATE.selectedLightId=hit.id; STATE.drag={type:'light'}; buildSidebar(); drawOverlay(); } }
  else if(STATE.tool==='focus'){ STATE.render.focusDepth = clamp(readPx(maps.depthCtx,uv.u,uv.v)[0] - readPx(maps.auxCtx,uv.u,uv.v)[1]*STATE.render.heightScale,0,1); STATE.drag={type:'focus'}; buildSidebar(); setDirty(); }
  else if(STATE.tool==='probe'){ updateProbe(uv.u,uv.v); }
  else if(STATE.tool==='pan'){ STATE.drag={type:'pan', sx:evt.clientX, sy:evt.clientY, px:STATE.viewport.panX, py:STATE.viewport.panY}; }
  else { STATE.drag={type:'paint', last:uv}; paintAt(uv.u,uv.v); drawOverlay(); }
}
function pointerMove(evt){ const uv=canvasUv(evt); if(!STATE.drag){ if(STATE.tool.startsWith('surf') || STATE.tool.startsWith('aux')){ STATE.drag={type:'hover',last:uv}; drawOverlay(); STATE.drag=null; } return; }
  if(STATE.drag.type==='direction'){ const l=getSelectedLight(); if(l){ const fine = evt.shiftKey ? 0.25 : 1.0; const rad=Math.atan2(uv.v-l.y, uv.u-l.x); l.dirX=Math.cos(rad)*fine + (l.dirX||0)*(1.0-fine); l.dirY=Math.sin(rad)*fine + (l.dirY||1)*(1.0-fine); buildSidebar(); setDirty(); drawOverlay(); } }
    else if(STATE.drag.type==='light'){ const l=getSelectedLight(); if(l){ l.x=uv.u; l.y=uv.v; setDirty(); drawOverlay(); buildSidebar(); } }
  else if(STATE.drag.type==='focus'){ STATE.render.focusDepth = clamp(readPx(maps.depthCtx,uv.u,uv.v)[0] - readPx(maps.auxCtx,uv.u,uv.v)[1]*STATE.render.heightScale,0,1); setDirty(); buildSidebar(); }
  else if(STATE.drag.type==='pan'){ STATE.viewport.panX = STATE.drag.px + (evt.clientX-STATE.drag.sx)/220; STATE.viewport.panY = STATE.drag.py + (evt.clientY-STATE.drag.sy)/220; applyStageTransform(); }
  else if(STATE.drag.type==='paint'){ STATE.drag.last=uv; paintAt(uv.u,uv.v); drawOverlay(); }
}
function pointerUp(){ STATE.drag=null; drawOverlay(); }
function wheel(evt){ evt.preventDefault(); const delta=Math.sign(evt.deltaY)*-1; if(STATE.tool==='moveLight'){ const l=getSelectedLight(); if(!l) return; if(evt.shiftKey){ if(l.type==='area' || l.type==='emissive'){ l.width=clamp((l.width||0.2)+delta*0.015,0.01,1); l.height=clamp((l.height||0.16)+delta*0.015,0.01,1); } else l.radius=clamp((l.radius||0.2)+delta*0.02,0.02,1.2); } else if(evt.altKey) l.intensity=clamp(l.intensity+delta*0.08,0,5); else l.z=clamp(l.z+delta*0.015,0,1); buildSidebar(); setDirty(); }
  else if(STATE.tool==='focus'){ if(evt.shiftKey) STATE.render.aperture=clamp(STATE.render.aperture+delta*0.01,0,0.35); else if(evt.altKey) STATE.render.bokehGain=clamp(STATE.render.bokehGain+delta*0.05,0,2.4); else STATE.render.focusDepth=clamp(STATE.render.focusDepth+delta*0.015,0,1); buildSidebar(); setDirty(); }
  else if(STATE.tool==='pan'){ STATE.viewport.zoom=clamp(STATE.viewport.zoom*(delta>0?1.08:0.92),0.5,2.4); applyStageTransform(); }
  else if(STATE.tool.startsWith('surf') || STATE.tool.startsWith('aux')){ if(evt.shiftKey) STATE.brush.hardness=clamp(STATE.brush.hardness+delta*0.03,0,1); else if(evt.altKey) STATE.brush.strength=clamp(STATE.brush.strength+delta*0.03,0,1); else STATE.brush.size=clamp(STATE.brush.size+delta*3,1,140); buildSidebar(); drawOverlay(); }
}
function applyStageTransform(){ el.stage.style.transform=`translate(${STATE.viewport.panX*40}px, ${STATE.viewport.panY*40}px) scale(${STATE.viewport.zoom})`; }
function downloadUrl(url, name){ const a=document.createElement('a'); a.href=url; a.download=name; a.click(); }
function downloadBlob(blob,name){ const a=document.createElement('a'); a.href=URL.createObjectURL(blob); a.download=name; a.click(); setTimeout(()=>URL.revokeObjectURL(a.href),1000); }

function cycleView(dir){ const i=VIEW_OPTIONS.indexOf(STATE.viewMode); STATE.viewMode=VIEW_OPTIONS[(i+dir+VIEW_OPTIONS.length)%VIEW_OPTIONS.length]; buildSidebar(); setDirty(); }
let soloBackup=null;
function toggleSolo(){ const l=getSelectedLight(); if(!l) return; if(!soloBackup){ soloBackup=STATE.lights.map(x=>({id:x.id, enabled:x.enabled})); STATE.lights.forEach(x=>x.enabled=(x.id===l.id)); } else { soloBackup.forEach(s=>{ const x=STATE.lights.find(l=>l.id===s.id); if(x) x.enabled=s.enabled; }); soloBackup=null; } buildSidebar(); setDirty(); }
function handleKey(e){ const tag=(e.target&&e.target.tagName||'').toLowerCase(); if(['input','select','textarea'].includes(tag)) return; if(e.ctrlKey && e.key.toLowerCase()==='d'){ e.preventDefault(); duplicateSelectedLight(); return; } if(e.key==='Delete'){ deleteSelectedLight(); return; } const k=e.key.toLowerCase(); if(k==='l') STATE.tool='moveLight'; else if(k==='r') STATE.tool='rotateLight'; else if(k==='f') STATE.tool='focus'; else if(k==='p') STATE.tool='probe'; else if(k==='h' || k===' ') STATE.tool='pan'; else if(k==='g'){ STATE.render.showHandles=!STATE.render.showHandles; drawOverlay(); } else if(k==='s') toggleSolo(); else if(k==='[') cycleView(-1); else if(k===']') cycleView(1); else if(k==='0'){ STATE.viewMode='final'; buildSidebar(); setDirty(); } else if(k>='1' && k<='5'){ const arr=['noirDefault','deskLampFocus','coldWindow','tvMood','previewSoft']; PRESETS[arr[parseInt(k)-1]](); buildSidebar(); setDirty(); } buildToolbar(); updateHelp(); }

async function start(){ setStatus('Ładowanie obrazów…'); const [base,depth,surface,aux,material,emissive] = await Promise.all([loadImage(BASE_IMAGE_URL), loadImage(DEPTH_IMAGE_URL), loadImage(SURFACE_IMAGE_URL), loadImage(AUX_IMAGE_URL), loadImage(MATERIAL_IMAGE_URL), loadImage(EMISSIVE_IMAGE_URL)]); assets.base=base; assets.depth=depth; assets.surface=surface; assets.aux=aux; assets.material=material; assets.emissive=emissive; Object.assign(maps, { ...(()=>{ const a=createCanvasFromImage(base); return {baseCanvas:a.canvas, baseCtx:a.ctx}; })(), ...(()=>{ const a=createCanvasFromImage(depth); return {depthCanvas:a.canvas, depthCtx:a.ctx}; })(), ...(()=>{ const a=createCanvasFromImage(surface); return {surfaceCanvas:a.canvas, surfaceCtx:a.ctx}; })(), ...(()=>{ const a=createCanvasFromImage(aux); return {auxCanvas:a.canvas, auxCtx:a.ctx}; })(), ...(()=>{ const a=createCanvasFromImage(material); return {materialCanvas:a.canvas, materialCtx:a.ctx}; })(), ...(()=>{ const a=createCanvasFromImage(emissive); return {emissiveCanvas:a.canvas, emissiveCtx:a.ctx}; })() });
  // ensure surface alpha is fully opaque
  let img = maps.surfaceCtx.getImageData(0,0,W,H); for(let i=3;i<img.data.length;i+=4) img.data[i]=255; maps.surfaceCtx.putImageData(img,0,0);
  initGl(); PRESETS.noirDefault(); buildToolbar(); buildSidebar(); updateHelp(); drawOverlay(); applyStageTransform(); ['pointerdown','wheel'].forEach(type=>{ el.sidebar.addEventListener(type,e=>e.stopPropagation(),true); el.toolbar.addEventListener(type,e=>e.stopPropagation(),true); }); el.glCanvas.addEventListener('pointerdown', pointerDown); window.addEventListener('pointermove', pointerMove); window.addEventListener('pointerup', pointerUp); el.glCanvas.addEventListener('wheel', wheel, {passive:false}); window.addEventListener('keydown', handleKey); setDirty(); setStatus('Gotowe'); }
start().catch(err=>{ console.error(err); setStatus('Błąd uruchomienia: '+err.message); });
})();
