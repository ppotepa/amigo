<script lang="ts">
  import type { BrushApplication, BrushDefinition, BrushMedium, NprStyleLayer, StrokeTool } from './contracts';

  export let brush: BrushDefinition | undefined;
  export let layer: NprStyleLayer;
  export let preview: string | undefined;
  export let pending = false;
  export let disabled = false;
  export let save: (brush: BrushDefinition, updateMatching: boolean) => void;

  let base = '';
  let draft: BrushDefinition | undefined;
  let updateMatching = false;
  let open = false;

  $: if (brush && JSON.stringify(brush) !== base) {
    base = JSON.stringify(brush);
    draft = structuredClone(brush);
  }
  $: dirty = !!draft && JSON.stringify(draft) !== base;

  const mediums: [BrushMedium, string][] = [
    ['ink', 'Atrament'], ['graphite', 'Grafit'], ['hatching', 'Kreskowanie'],
    ['flat-fill', 'Wypełnienie'], ['watercolour-wash', 'Akwarela'],
  ];
  const tools: [StrokeTool, string][] = [
    ['pencil', 'Ołówek'], ['fineliner', 'Cienkopis'], ['nib', 'Pióro'], ['brush', 'Pędzel'],
  ];
  const fields: [keyof BrushDefinition, string, number, number, number][] = [
    ['width', 'Szerokość', .1, 12, .1], ['pressure_profile', 'Nacisk', 0, 1, .01],
    ['taper', 'Zwężenie końców', 0, 1, .01], ['softness', 'Miękkość', 0, 1, .01],
    ['spacing', 'Rozstaw', .01, 4, .01], ['irregularity', 'Nieregularność', 0, 1, .01],
    ['correction', 'Korekcja', 0, 1, .01], ['dryness', 'Suchość', 0, 1, .01],
  ];
  const applications: [BrushApplication, string][] = [['stroke', 'Linie'], ['surface', 'Farby']];
  const mediumLabel = (value: BrushMedium) => mediums.find(([id]) => id === value)?.[1] ?? value;
  const applicationLabel = (value: BrushApplication) => applications.find(([id]) => id === value)?.[1] ?? value;
  const number = (event: Event) => Number((event.currentTarget as HTMLInputElement).value);
  function update(patch: Partial<BrushDefinition>) { if (draft) draft = { ...draft, ...patch }; }
  function setField(field: keyof BrushDefinition, value: number) { update({ [field]: value }); }
  function setPaint(field: 'wash' | 'granulation', value: number) {
    if (draft?.paint) update({ paint: { ...draft.paint, [field]: value } });
  }
  function toggleApplication(application: BrushApplication, enabled: boolean) {
    if (!draft) return;
    const next = enabled ? [...new Set([...draft.applications, application])] : draft.applications.filter((item) => item !== application);
    if (next.length) update({ applications: next });
  }
  function commit() {
    if (!draft || !dirty || pending || !draft.name.trim()) return;
    save({ ...draft, name: draft.name.trim() }, updateMatching);
  }
</script>

{#if brush && draft}
  <section class="lab">
    <header>
      <h2>Brush Lab</h2>
      <button type="button" disabled={disabled} aria-expanded={open} onclick={() => open = !open}>{open ? 'Zwiń' : 'Edytuj'}</button>
    </header>
    <div class="brush-summary">
      {#if preview}<img src={preview} alt={`Podgląd pędzla: ${brush.name}`} />{:else}<span class="missing">—</span>{/if}
      <span><strong>{brush.name}</strong><small>{mediumLabel(brush.medium)}</small><small>{brush.applications.map(applicationLabel).join(' + ')}</small></span>
    </div>
    {#if open}
      <form onsubmit={(event) => { event.preventDefault(); commit(); }}>
        <fieldset disabled={disabled}>
        <label>Nazwa pędzla<input maxlength="80" value={draft.name} oninput={(event) => update({ name: event.currentTarget.value })} /></label>
        <label>Rodzaj<select value={draft.medium} onchange={(event) => update({ medium: event.currentTarget.value as BrushMedium })}>{#each mediums as [id, label]}<option value={id}>{label}</option>{/each}</select></label>
        <label>Narzędzie<select value={draft.tool ?? ''} onchange={(event) => update({ tool: (event.currentTarget.value || undefined) as StrokeTool | undefined })}><option value="">Domyślne</option>{#each tools as [id, label]}<option value={id}>{label}</option>{/each}</select></label>
        <fieldset><legend>Zastosowanie</legend>{#each applications as [id, label]}<label class="check"><input type="checkbox" checked={draft.applications.includes(id)} onchange={(event) => toggleApplication(id, event.currentTarget.checked)} />{label}</label>{/each}</fieldset>
        {#each fields as [field, label, min, max, step]}<label>{label}<output>{Number(draft[field]).toFixed(2)}</output><input type="range" {min} {max} {step} value={Number(draft[field])} oninput={(event) => setField(field, number(event))} /></label>{/each}
        {#if draft.paint}<h3>Farba</h3><label>Podmalówka<output>{draft.paint.wash.toFixed(2)}</output><input type="range" min="0" max="2" step=".02" value={draft.paint.wash} oninput={(event) => setPaint('wash', number(event))} /></label><label>Granulacja<output>{draft.paint.granulation.toFixed(2)}</output><input type="range" min="0" max="1" step=".01" value={draft.paint.granulation} oninput={(event) => setPaint('granulation', number(event))} /></label>{/if}
        <label class="check"><input type="checkbox" bind:checked={updateMatching} />Przypnij nową wersję do wszystkich pozycji używających starej</label>
        <small>Zapisywana jest nowa wersja. Bieżąca pozycja zostanie do niej przypięta.</small>
        <footer><button type="submit" class="primary" disabled={!dirty || pending || disabled || !draft.name.trim()}>Zapisz wersję</button></footer>
        </fieldset>
      </form>
    {/if}
  </section>
{:else if layer.source !== 'paper'}
  <section class="lab"><h2>Brush Lab</h2><p>Ta pozycja nie ma jeszcze pędzla. Wybierz zapisane ustawienia powyżej.</p></section>
{/if}

<style>
  .lab{padding:12px;border-top:1px solid #48483d}.lab header{display:flex;align-items:center;justify-content:space-between;gap:8px}.lab h2{margin:0;font-size:13px}.brush-summary{display:flex;align-items:center;gap:9px;margin-top:9px}.brush-summary img,.missing{width:58px;height:48px;object-fit:contain;border:1px solid #44483b;border-radius:4px;background:#11130f}.missing{display:grid;place-items:center;color:#999}.brush-summary strong,.brush-summary small{display:block}.brush-summary small,p{color:#999e8e;font-size:10px}.lab label{display:grid;gap:4px;margin:9px 0;color:#c8c6be;font-size:11px}.lab output{float:right;color:#d3aa65}.lab input,.lab select{width:100%;font:inherit;accent-color:#c69b55}.lab input:not([type=range]):not([type=checkbox]),.lab select{padding:6px;border:1px solid #484942;border-radius:4px;background:#151613;color:#e8e5df}.lab fieldset{border:1px solid #3e403a;border-radius:4px;padding:6px}.lab legend{color:#c8c6be;font-size:11px}.check{display:flex!important;align-items:center;gap:6px}.check input{width:auto}.lab h3{font-size:11px;color:#e2c794}.lab footer{display:flex;justify-content:flex-end;margin-top:10px}
</style>
