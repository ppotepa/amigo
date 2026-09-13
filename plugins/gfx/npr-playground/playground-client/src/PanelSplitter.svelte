<script lang="ts">
  import { panelWidthFromDelta, panelWidthFromKey, type PanelSide } from './layout';

  export let side: PanelSide;
  export let value: number;
  export let minimum: number;
  export let maximum: number;
  export let resize: (value: number) => void;

  let startPointer = 0;
  let startValue = 0;
  let dragging = false;

  function pointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    startPointer = event.clientX;
    startValue = value;
    dragging = true;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function pointerMove(event: PointerEvent) {
    if (dragging) resize(panelWidthFromDelta(side, startValue, event.clientX - startPointer, minimum, maximum));
  }

  function pointerUp(event: PointerEvent) {
    dragging = false;
    (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
  }

  function keydown(event: KeyboardEvent) {
    const next = panelWidthFromKey(side, event.key, value, minimum, maximum);
    if (next === undefined) return;
    event.preventDefault();
    resize(next);
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex a11y_no_noninteractive_element_interactions -->
<div class:dragging class="splitter" role="separator" tabindex="0"
  aria-orientation="vertical" aria-valuemin={minimum} aria-valuemax={maximum} aria-valuenow={value}
  aria-label={side === 'left' ? 'Szerokość panelu modeli' : 'Szerokość panelu presetu'}
  onpointerdown={pointerDown} onpointermove={pointerMove} onpointerup={pointerUp}
  onpointercancel={() => dragging = false} onkeydown={keydown}></div>

<style>
  .splitter{position:relative;z-index:2;width:8px;height:100%;background:#171916;border-inline:1px solid #363832;cursor:col-resize;touch-action:none}
  .splitter::after{content:'';position:absolute;left:3px;top:calc(50% - 18px);height:36px;border-left:1px dotted #77786b;opacity:.65}
  .splitter:hover,.splitter:focus-visible,.splitter.dragging{background:#39352b;border-color:#c69b55}
  .splitter:focus-visible{outline:2px solid #d3aa65;outline-offset:-2px}
  @media(max-width:760px){.splitter{display:none}}
</style>
