<script lang="ts">
  import type { CameraMode } from './camera-input';
  import type { PresentationMode } from './contracts';

  export let canvas: HTMLCanvasElement;
  export let rgbaCanvas: HTMLCanvasElement;
  export let viewport: HTMLDivElement;
  export let activeMode: PresentationMode | null = null;
  export let cameraMode: CameraMode = 'select';
  export let renderSize: [number, number] = [1, 1];
  export let fps = 0;
  export let ready = false;
  export let message = '';
  export let selectMode: () => void;
  export let orbitMode: () => void;
  export let toggleCoverage: () => void;
  export let coverageActive = false;
  export let pointerDown: (event: PointerEvent) => void;
  export let pointerMove: (event: PointerEvent) => void;
  export let pointerUp: (event: PointerEvent) => void;
  export let pointerCancel: () => void;
  export let wheel: (event: WheelEvent) => void;
</script>

<section class="drawing">
  <nav aria-label="Viewport tools">
    <span>Paper viewport · orbit, pan and zoom</span><i></i>
    <button class:active={cameraMode === 'select'} onclick={selectMode}>Select</button>
    <button class:active={cameraMode === 'orbit'} onclick={orbitMode}>Orbit</button>
    <button class:active={coverageActive} onclick={toggleCoverage}>Coverage</button>
  </nav>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div class="viewport" bind:this={viewport} tabindex="0" role="application" aria-label="NPR drawing viewport"
    oncontextmenu={(event) => event.preventDefault()} onpointerdown={(event) => { event.preventDefault(); pointerDown(event); }} onpointermove={(event) => { event.preventDefault(); pointerMove(event); }}
    onpointerup={(event) => { event.preventDefault(); pointerUp(event); }} onpointercancel={pointerCancel} onlostpointercapture={pointerCancel} onwheel={(event) => { event.preventDefault(); wheel(event); }}>
    <canvas bind:this={canvas} class:hidden={activeMode === 'local_rgba'}></canvas>
    <canvas bind:this={rgbaCanvas} class:hidden={activeMode !== 'local_rgba'}></canvas>
    {#if !ready}<div class="empty">Preparing paper and model…</div>{/if}
    {#if message}<div class="message">{message}</div>{/if}
    <small class="hud">{activeMode ? 'Drawing ready' : 'Preparing render'} · {fps} FPS · {renderSize[0]}×{renderSize[1]}</small>
  </div>
</section>

<style>
  .drawing{display:grid;grid-template-rows:42px minmax(0,1fr);min-width:0;min-height:0;background:#141512}
  nav{display:flex;align-items:center;gap:7px;padding:6px 10px;border-bottom:1px solid #30312d;color:#989a90;font-size:11px}
  nav i{flex:1}button{font:inherit;border:1px solid #41413e;border-radius:6px;background:#272826;color:inherit;padding:4px 7px;cursor:pointer}
  button:hover{background:#393a36}.active{border-color:#b58d4c;background:#51432a;color:#fff8e9}
  .viewport{position:relative;min-height:0;overflow:hidden;outline:none;touch-action:none;cursor:grab}.viewport:active{cursor:grabbing}.viewport canvas{position:absolute;inset:0;display:block;width:100%;height:100%;object-fit:contain;touch-action:none;pointer-events:none}
  .hidden{visibility:hidden}.empty{position:absolute;inset:0;display:grid;place-items:center;color:#a6a89b;pointer-events:none}.message{position:absolute;top:12px;right:12px;padding:8px;background:#321f1d;color:#efad9f;pointer-events:none}
  .hud{position:absolute;bottom:12px;left:12px;padding:5px 8px;border-radius:4px;background:#12130fcc;color:#bbb9ad;pointer-events:none}
</style>
