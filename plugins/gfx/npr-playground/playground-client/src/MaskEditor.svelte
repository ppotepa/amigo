<script lang="ts">
  import type { CoverageMask } from './contracts';
  export let value: CoverageMask;
  export let change: (next: CoverageMask) => void;
  const children = (mask: CoverageMask): CoverageMask[] =>
    mask.kind === 'multiply' ? mask.masks : [];
  function setKind(kind: CoverageMask['kind']) {
    if (kind === 'none') change({ kind });
    else if (kind === 'tone-range' || kind === 'height') change({ kind, min: 0, max: 1, invert: false });
    else if (kind === 'normal-direction') change({ kind, direction: [0, 0, 1], threshold: 0.5, invert: false });
    else if (kind === 'noise') change({ kind, amount: 0.2, seed: 1, invert: false });
    else change({ kind, masks: [] });
  }
  function updateNumber(field: 'min' | 'max' | 'threshold' | 'amount' | 'seed', event: Event) {
    change({ ...value, [field]: Number((event.currentTarget as HTMLInputElement).value) } as CoverageMask);
  }
</script>

<div class="mask">
  <label>Coverage mask<select value={value.kind} onchange={(event) => setKind((event.currentTarget as HTMLSelectElement).value as CoverageMask['kind'])}>
    <option value="none">Brak</option><option value="tone-range">Oświetlenie powierzchni</option><option value="height">Wysokość na modelu</option>
    <option value="normal-direction">Kierunek normalnej powierzchni</option><option value="noise">Szum</option><option value="multiply">Iloczyn masek</option>
  </select></label>
  {#if value.kind === 'tone-range' || value.kind === 'height'}
    <label>Min<input type="number" min="0" max="1" step=".05" value={value.min} onchange={(event) => updateNumber('min', event)} /></label>
    <label>Max<input type="number" min="0" max="1" step=".05" value={value.max} onchange={(event) => updateNumber('max', event)} /></label>
    <label class="check"><input type="checkbox" checked={value.invert} onchange={(event) => change({ ...value, invert: (event.currentTarget as HTMLInputElement).checked })} /> Invert</label>
  {:else if value.kind === 'normal-direction'}
    <p>Kierunek jest liczony w układzie modelu, niezależnie od kamery.</p>
    <label>Direction X/Y/Z<div class="triple">{#each value.direction as axis, index}<input type="number" min="-1" max="1" step=".1" value={axis} onchange={(event) => { const direction = [...value.direction] as [number, number, number]; direction[index] = Number((event.currentTarget as HTMLInputElement).value); change({ ...value, direction }); }} />{/each}</div></label>
    <label>Threshold<input type="number" min="-1" max="1" step=".05" value={value.threshold} onchange={(event) => updateNumber('threshold', event)} /></label>
    <label class="check"><input type="checkbox" checked={value.invert} onchange={(event) => change({ ...value, invert: (event.currentTarget as HTMLInputElement).checked })} /> Invert</label>
  {:else if value.kind === 'noise'}
    <label>Amount<input type="number" min="0" max="1" step=".05" value={value.amount} onchange={(event) => updateNumber('amount', event)} /></label>
    <label>Seed<input type="number" min="0" step="1" value={value.seed} onchange={(event) => updateNumber('seed', event)} /></label>
    <label class="check"><input type="checkbox" checked={value.invert} onchange={(event) => change({ ...value, invert: (event.currentTarget as HTMLInputElement).checked })} /> Invert</label>
  {:else if value.kind === 'multiply'}
    {#each children(value) as child, index}
      <div class="nested"><svelte:self value={child} change={(next: CoverageMask) => { const nextChildren = children(value).slice(); nextChildren[index] = next; change({ kind: 'multiply', masks: nextChildren }); }} /><button type="button" aria-label={`Usuń maskę ${index + 1}`} onclick={() => change({ kind: 'multiply', masks: children(value).filter((_, i) => i !== index) })}>Usuń maskę</button></div>
    {/each}
    <button type="button" onclick={() => change({ kind: 'multiply', masks: [...children(value), { kind: 'none' }] })}>Add mask</button>
  {/if}
</div>

<style>
  .mask{padding:7px;border:1px solid #3e403a;border-radius:5px;background:#191a17}.mask label{display:grid;gap:3px;margin:6px 0;color:#c8c6be;font-size:11px}
  select,input{width:100%;font:inherit;border:1px solid #484942;border-radius:4px;background:#10110f;color:#e8e5df;padding:4px}.check{display:flex!important;grid-template-columns:auto 1fr!important;align-items:center;gap:5px}.check input{width:auto}.triple{display:grid;grid-template-columns:repeat(3,1fr);gap:4px}.nested{margin:6px 0 6px 8px;border-left:2px solid #51432a;padding-left:6px}button{font:inherit;border:1px solid #41413e;border-radius:5px;background:#272826;color:inherit;padding:5px 7px;cursor:pointer}
</style>
