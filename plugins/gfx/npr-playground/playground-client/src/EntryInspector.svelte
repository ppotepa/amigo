<script lang="ts">
  import { onDestroy } from 'svelte';
  import MaskEditor from './MaskEditor.svelte';
  import type { BrushInstance, BrushLibrary, ComicInk, NprStyleLayer, StrokeTool } from './contracts';
  import { brushFor, colorHex, effectiveTool, hexColor, isPaint, latestAppearances, sourceNames, toolNames, useAppearance } from './drawing';
  export let layer: NprStyleLayer;
  export let library: BrushLibrary;
  export let style: ComicInk;
  export let model: string;
  export let locked = false;
  export let pending = false;
  export let image: string | undefined;
  export let imageError: string | undefined;
  export let brushPreviews: Record<string, string | null> = {};
  export let apply: (value: NprStyleLayer) => void;
  export let previewChange: (value: NprStyleLayer) => void;
  export let cancel: () => void;
  export let onDirty: (value: boolean) => void;
  export let saveAppearance: (value: NprStyleLayer, name: string, updateMatching: boolean) => void;
  let base = JSON.stringify(layer);
  let draft = structuredClone(layer);
  let appearanceName = '';
  let updateMatching = false;
  let previewTimer: ReturnType<typeof setTimeout> | undefined;
  $: if (JSON.stringify(layer) !== base) { base = JSON.stringify(layer); draft = structuredClone(layer); }
  $: dirty = JSON.stringify(draft) !== base;
  $: onDirty(dirty);
  $: brush = brushFor(draft, library);
  $: templates = latestAppearances(library, draft.source);
  $: paint = draft.paint ?? brush?.paint ?? { wash: 1, granulation: 0 };
  $: hatch = draft.hatch ?? { density: 1, spacing: 1 };
  type AppearanceField = 'width' | 'taper' | 'softness' | 'pressure_profile' | 'irregularity' | 'correction' | 'dryness' | 'spacing';
  const appearanceFields: [AppearanceField, string, number, number, number][] = [
    ['width','Szerokość kreski',0.1,12,0.1], ['pressure_profile','Nacisk',0,1,0.01],
    ['taper','Zwężenie końców',0,1,0.01], ['softness','Miękkość',0,1,0.01],
    ['irregularity','Nieregularność gestu',0,1,0.01], ['correction','Korekcja gestu',0,1,0.01],
    ['dryness','Suchość',0,1,0.01], ['spacing','Rozstaw kontaktu narzędzia',0.01,4,0.01],
  ];
  const number = (event: Event) => Number((event.currentTarget as HTMLInputElement).value);
  function change(next: NprStyleLayer) {
    draft = next;
    clearTimeout(previewTimer);
    if (!locked && draft.label.trim()) previewTimer = setTimeout(() => previewChange(structuredClone(draft)), 100);
  }
  function appearanceValue(field: AppearanceField): number { return draft.brush?.[field] ?? brush?.[field] ?? (field === 'width' ? style.outline_width : 0); }
  function setAppearance(field: AppearanceField, value: number) {
    let instance: BrushInstance | undefined = draft.brush;
    if (!instance && templates[0]) instance = { brush: { id: templates[0].id, version: templates[0].version } };
    if (instance) change({ ...draft, brush: { ...instance, [field]: value } });
  }
  function brushPreview(id: string, version: number) { return brushPreviews[`${id}@${version}`]; }
  function discard() { clearTimeout(previewTimer); draft = structuredClone(layer); cancel(); }
  function commit() { clearTimeout(previewTimer); if (dirty && draft.label.trim() && !locked && !pending) apply(structuredClone(draft)); }
  function keydown(event: KeyboardEvent) {
    if (!(event.target as HTMLElement)?.closest('.inspector')) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); discard(); }
    if (!event.repeat && (event.ctrlKey || event.metaKey) && event.key === 'Enter') { event.preventDefault(); commit(); }
  }
  onDestroy(() => clearTimeout(previewTimer));
</script>

<svelte:window onkeydown={keydown} />
<section class="inspector" aria-label="Właściwości pozycji">
  <header><h2>{sourceNames[layer.source]}</h2><small>{dirty ? 'Podgląd zmian' : 'Właściwości'}</small></header>
  {#if locked}<p class="warning">Pozycja jest zablokowana. Odblokuj ją, aby edytować.</p>{/if}
  <form onsubmit={(event) => { event.preventDefault(); commit(); }}>
    <fieldset disabled={locked}>
      <label>Nazwa<input maxlength="80" value={draft.label} oninput={(event) => change({ ...draft, label: event.currentTarget.value })} /></label>
      {#if image}<img class="sample" src={image} alt={`Próbka ustawień: ${draft.label}`} />{:else}<p>{imageError ?? 'Przygotowywanie próbki…'}</p>{/if}
      <p class="hint">Próbka narzędzia powyżej. Pełny efekt zmian zobaczysz na modelu w viewportcie.</p>
      {#if layer.source !== 'paper'}
        <details>
          <summary>Użyj zapisanych ustawień…</summary>
          <label>Wygląd narzędzia<select aria-label="Wybierz ustawienia narzędzia" value="" onchange={(event) => { const selected = templates.find((item) => item.id === event.currentTarget.value); if (selected) change(useAppearance(draft,selected)); }}>
            <option value="">Wybierz ustawienia</option>{#each templates as item}<option value={item.id}>{item.name}</option>{/each}
          </select></label>
          <div class="brush-picker" aria-label="Podglądy pędzli">
            {#each templates as item}
              <button type="button" class:chosen={brush?.id === item.id && brush?.version === item.version} onclick={() => change(useAppearance(draft,item))}>
                {#if brushPreview(item.id,item.version)}<img src={brushPreview(item.id,item.version) ?? undefined} alt="" />{:else}<span>—</span>{/if}
                <small>{item.name}</small>
              </button>
            {/each}
          </div>
          {#if draft.brush}<button type="button" class="remove-brush" onclick={() => change({ ...draft, brush: undefined })}>Usuń przypisanie pędzla</button>{/if}
          <p class="hint">Zmienia tylko tę pozycję. Źródło geometrii pozostaje bez zmian.</p>
        </details>
      {/if}
      {#if !isPaint(layer.source) && layer.source !== 'paper'}
        <h3>Źródło linii</h3>
        {#if layer.source === 'construction'}<p>Przebieg pochodzi z punktów konstrukcyjnych zapisanych na powierzchni modelu.</p>{/if}
        {#if layer.source === 'creases'}<label>Próg ostrej krawędzi <output>{Math.round((draft.line?.crease_angle ?? style.crease_angle)*180/Math.PI)}°</output><input type="range" min="0" max="3.14" step=".01" value={draft.line?.crease_angle ?? style.crease_angle} oninput={(event) => change({ ...draft, line: { ...draft.line, crease_angle: number(event) } })} /></label>{/if}
        {#if layer.source === 'creases'}<label>Minimalna długość <output>{draft.line?.min_length_pixels ?? style.min_crease_length_pixels} px</output><input type="range" min="0" max="100" step="1" value={draft.line?.min_length_pixels ?? style.min_crease_length_pixels} oninput={(event) => change({ ...draft, line: { ...draft.line, min_length_pixels: number(event) } })} /></label>{/if}
        {#if layer.source === 'shadow-hatch'}
          <label>Gęstość <output>{Math.round((draft.hatch?.density ?? 1)*100)}%</output><input type="range" min="0" max="1" step=".01" value={draft.hatch?.density ?? 1} oninput={(event) => change({ ...draft, hatch: { ...hatch,density:number(event) } })} /></label>
          <label>Odstęp kresek <output>×{(draft.hatch?.spacing ?? 1).toFixed(2)}</output><input type="range" min=".25" max="4" step=".05" value={draft.hatch?.spacing ?? 1} oninput={(event) => change({ ...draft, hatch: { ...hatch,spacing:number(event) } })} /></label>
          <label>Kierunek <output>{draft.hatch?.angle ?? style.hatching_angle}°</output><input type="range" min="-180" max="180" step="1" value={draft.hatch?.angle ?? style.hatching_angle} oninput={(event) => change({ ...draft, hatch: { ...hatch,angle:number(event) } })} /></label>
          <label>Kreskowanie krzyżowe<input type="range" min="0" max="1" step=".01" value={draft.hatch?.cross ?? style.hatching_cross} oninput={(event) => change({ ...draft, hatch: { ...hatch,cross:number(event) } })} /></label>
        {/if}
        <label>Uproszczenie przebiegu<input type="range" min="0" max="1" step=".01" value={draft.line?.simplification ?? style.gesture_simplification} oninput={(event) => change({ ...draft, line: { ...draft.line, simplification:number(event) } })} /></label>
        <h3>Narzędzie i ślad</h3>
        <label>Narzędzie<select value={effectiveTool(draft,brush)} onchange={(event) => change({ ...draft, tool: event.currentTarget.value as StrokeTool })}>{#each Object.entries(toolNames) as [id,label]}<option value={id}>{label}</option>{/each}</select></label>
        {#each appearanceFields as [field,label,min,max,step]}<label>{label}<output>{appearanceValue(field).toFixed(2)}</output><input type="range" aria-label={label} disabled={(field === 'pressure_profile' && effectiveTool(draft,brush) === 'fineliner') || (field === 'correction' && appearanceValue('irregularity') === 0) || (field === 'spacing' && appearanceValue('dryness') === 0)} {min} {max} {step} value={appearanceValue(field)} oninput={(event) => setAppearance(field,number(event))} /></label>{/each}
      {:else if layer.source === 'wash'}
        <h3>Farba</h3>
        <label>Nasycenie podmalówki <output>{paint.wash.toFixed(2)}</output><input type="range" min="0" max="2" step=".02" value={paint.wash} oninput={(event) => change({ ...draft, paint: { ...paint, wash: number(event) } })} /></label>
        <label>Granulacja<input type="range" min="0" max="1" step=".01" value={paint.granulation} oninput={(event) => change({ ...draft, paint: { ...paint, granulation:number(event) } })} /></label>
      {/if}
      <h3>Kolor i nakładanie</h3>
      <label>Kolor<select value={draft.color_source.kind} onchange={(event) => change({ ...draft, color_source: event.currentTarget.value === 'constant' ? { kind:'constant', color: layer.source === 'paper' ? style.paper : style.ink } : { kind: event.currentTarget.value as 'style-palette' | 'model-base-color' } })}><option value="style-palette">Paleta presetu</option><option value="constant">Własny kolor</option>{#if layer.source !== 'paper'}<option value="model-base-color">Kolor modelu</option>{/if}</select></label>
      {#if draft.color_source.kind === 'constant'}<label>Własny kolor<input type="color" value={colorHex(draft.color_source.color)} oninput={(event) => change({ ...draft, color_source: { kind:'constant',color:hexColor(event.currentTarget.value) } })} /></label>{/if}
      {#if layer.source !== 'paper'}
        <label>Krycie <output>{Math.round(draft.opacity*100)}%</output><input type="range" min="0" max="1" step=".01" value={draft.opacity} oninput={(event) => change({ ...draft, opacity:number(event) })} /></label>
        <label>Mieszanie<select value={draft.blend} onchange={(event) => change({ ...draft, blend:event.currentTarget.value as NprStyleLayer['blend'] })}><option value="normal">Normalne</option><option value="multiply">Mnożenie</option><option value="screen">Rozjaśnianie</option></select></label>
        <details><summary>Obszar i maska</summary>
          <label>Obszar modelu<select value={draft.target.kind} onchange={(event) => change({ ...draft, target: event.currentTarget.value === 'objects' ? { kind:'objects',objects:[model] } : event.currentTarget.value === 'surface-features' ? { kind:'surface-features',features:[layer.source] } : { kind:'all' } })}><option value="all">Cała geometria</option><option value="objects">Aktywny model</option><option value="surface-features">Rodzaje geometrii</option></select></label>
          {#if draft.target.kind === 'surface-features'}
            {#each Object.entries(sourceNames).filter(([id]) => id !== 'paper') as [id,label]}<label class="check"><input type="checkbox" checked={draft.target.features.includes(id)} onchange={(event) => { if (draft.target.kind !== 'surface-features') return; const features = event.currentTarget.checked ? [...draft.target.features,id] : draft.target.features.filter((item) => item !== id); change({ ...draft,target:{ kind:'surface-features',features } }); }} />{label}</label>{/each}
          {/if}
          <MaskEditor value={draft.mask} change={(mask) => change({ ...draft,mask })} />
        </details>
        <details><summary>Zapisz ustawienia narzędzia…</summary>
          <label>Nazwa ustawień<input maxlength="80" bind:value={appearanceName} placeholder={brush?.name ?? 'Moje narzędzie'} /></label>
          <label class="check"><input type="checkbox" bind:checked={updateMatching} />Zaktualizuj też inne pozycje tego rysunku używające tych ustawień</label>
          <button type="button" disabled={pending || !appearanceName.trim()} onclick={() => { clearTimeout(previewTimer); saveAppearance(draft,appearanceName,updateMatching); }}>Zapisz ustawienia</button>
        </details>
      {/if}
    </fieldset>
    <footer><button type="button" disabled={!dirty} onclick={discard}>Anuluj</button><button class="primary" type="submit" disabled={!dirty || locked || pending || !draft.label.trim()}>Zastosuj</button></footer>
  </form>
</section>

<style>
  .inspector{padding:14px 12px;border-top:1px solid #48483d}header{display:flex;align-items:center;justify-content:space-between;gap:8px}h2{font-size:14px;margin:0}h3{font-size:12px;margin:18px 0 8px;color:#e2c794}label{display:grid;gap:4px;margin:10px 0;color:#c8c6be;font-size:11px}output{grid-row:1;justify-self:end;color:#d3aa65}input,select{width:100%;font:inherit;accent-color:#c69b55}input:not([type=range]):not([type=checkbox]),select{padding:6px;border:1px solid #484942;border-radius:4px;background:#151613;color:#e8e5df}fieldset{padding:0;border:0;margin:0;min-width:0}fieldset:disabled{opacity:.6}.sample{display:block;width:100%;height:105px;object-fit:contain;border:1px solid #44483b;border-radius:6px;background:#131510}.hint,small,p{color:#999e8e;font-size:10px}.warning{padding:8px;color:#f1c17d;background:#493520}.check{display:flex;align-items:start;gap:6px}.check input{width:auto}.brush-picker{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:6px;margin:8px 0}.brush-picker button{display:flex;align-items:center;gap:5px;min-width:0;padding:4px;text-align:left}.brush-picker img,.brush-picker button>span{width:34px;height:28px;object-fit:contain;background:#11130f;flex-shrink:0}.brush-picker button>span{display:grid;place-items:center;color:#999}.brush-picker small{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.brush-picker .chosen{border-color:#c69b55;background:#302b21}.remove-brush{font-size:10px;padding:4px}details{border:1px solid #3e403a;padding:9px;margin:12px 0;border-radius:5px}summary{cursor:pointer;font-size:11px;color:#ddd4bd}footer{position:sticky;bottom:0;display:flex;justify-content:flex-end;gap:7px;background:#20211ef5;padding:12px 0 2px;border-top:1px solid #3e403a}.primary{border-color:#b58d4c;background:#51432a}
</style>
