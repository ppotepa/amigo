<script lang="ts">
  import type { BrushLibrary, NprStyleLayer } from './contracts';
  import { appearanceLabel, category, sourceNames } from './drawing';
  export let layers: NprStyleLayer[] = [];
  export let library: BrushLibrary;
  export let previews: Record<string, string> = {};
  export let brushPreviews: Record<string, string | null> = {};
  export let locked: string[] = [];
  export let selectedLayerId: string | undefined;
  export let selectLayer: (id: string) => void;
  export let toggle: (layer: NprStyleLayer) => void;
  export let disabled = false;
  let collapsed: string[] = [];
  let tree: HTMLDivElement;
  function keydown(event: KeyboardEvent) {
    if (!['ArrowDown','ArrowUp','Home','End'].includes(event.key)) return;
    const items = Array.from(tree.querySelectorAll<HTMLButtonElement>('[data-entry]'));
    const index = items.indexOf(event.target as HTMLButtonElement);
    if (index < 0) return;
    event.preventDefault();
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? items.length-1 : Math.max(0,Math.min(items.length-1,index+(event.key==='ArrowDown'?1:-1)));
    items[next]?.focus();
  }
</script>

<div class="tree" bind:this={tree} role="group" aria-label="Zawartość presetu">
  {#each ['Linie','Farby','Papier'] as group}
    {@const entries = layers.filter((layer) => category(layer.source) === group)}
    <section>
      <h3><button class="heading" aria-expanded={!collapsed.includes(group)} onclick={() => collapsed = collapsed.includes(group) ? collapsed.filter((item) => item !== group) : [...collapsed,group]}><span>{collapsed.includes(group) ? '▸' : '▾'} {group}</span><small>{entries.length}</small></button></h3>
      {#if !collapsed.includes(group)}
        <ul>
          {#each entries as layer (layer.id)}
            <li class:active={selectedLayerId === layer.id} class:hidden={!layer.enabled}>
              <button class="entry" data-entry={layer.id} aria-pressed={selectedLayerId === layer.id} onkeydown={keydown} onclick={() => selectLayer(layer.id)}>
                {#if previews[layer.id]}<img src={previews[layer.id]} alt={`Próbka: ${layer.label}`} />{:else}<span class="missing" title="Próbka jest niedostępna">—</span>{/if}
                <span class="name"><strong>{layer.label}</strong><small>{appearanceLabel(layer,library)}{locked.includes(layer.id) ? ' · zablokowane' : ''}</small><small>{sourceNames[layer.source]}</small></span>
              </button>
              {#if layer.source !== 'paper'}<input type="checkbox" aria-label={`Widoczność: ${layer.label}`} checked={layer.enabled} disabled={disabled || locked.includes(layer.id)} onchange={() => toggle(layer)} />{/if}
            </li>
            {#if layer.source !== 'paper' && layer.brush}
              {@const reference = layer.brush.brush}
              {@const preview = brushPreviews[`${reference.id}@${reference.version}`]}
              <li class="brush-row">
                {#if preview}<img src={preview} alt={`Podgląd pędzla dla ${layer.label}`} />{:else}<span class="missing">—</span>{/if}
                <span><small>Pędzel</small><strong>{library.brushes[reference.id]?.find((item) => item.version === reference.version)?.name ?? 'Brak definicji'}</strong></span>
              </li>
            {/if}
          {:else}<li class="empty">{group === 'Farby' ? 'Opcjonalne — dodaj farbę poniżej.' : 'Brak pozycji.'}</li>{/each}
        </ul>
      {/if}
    </section>
  {/each}
</div>

<style>
  .tree{padding:0 12px 12px}.tree section{margin:12px 0 18px}h3{margin:0 0 7px}.heading{display:flex;align-items:center;justify-content:space-between;width:100%;border:0;background:none;padding:5px 0;font-weight:600;text-align:left}ul{list-style:none;margin:0 0 0 13px;padding:0 0 0 10px;border-left:1px solid #494a42;display:grid;gap:9px}li{display:flex;align-items:center;gap:7px;border:1px solid #3e403a;border-radius:6px;background:#191a17;padding:3px 8px 3px 3px;min-width:0}li.active{border-color:#c69b55;background:#302b21}li.hidden .entry{opacity:.5}.entry{display:flex;align-items:center;gap:9px;width:100%;min-width:0;text-align:left;border:0;background:none;padding:3px}.entry img,.missing{width:54px;height:48px;object-fit:contain;border-radius:4px;background:#10120e;flex-shrink:0}.missing{display:grid;place-items:center;color:#999}.name{min-width:0}.name strong,.name small{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.name strong{font-size:12px}.name small,small,.empty{font-size:10px;color:#aaa99f}.name small+small{margin-top:2px;color:#818478}.brush-row{margin-left:28px;border:0;border-left:1px dotted #55594d;border-radius:0;background:transparent;padding-left:8px}.brush-row img,.brush-row .missing{width:38px;height:32px}.brush-row span:last-child{min-width:0}.brush-row strong,.brush-row small{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.empty{padding:10px;border-style:dashed}input{accent-color:#c69b55}
</style>
