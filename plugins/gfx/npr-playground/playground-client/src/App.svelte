<script lang="ts">
  import { onMount } from 'svelte';
  import { companionHost, type CompanionHost } from './companion-host';
  import EntryInspector from './EntryInspector.svelte';
  import PresetTree from './PresetTree.svelte';
  import ViewportPane from './ViewportPane.svelte';
  import PlaybackPanel from './PlaybackPanel.svelte';
  import PanelSplitter from './PanelSplitter.svelte';
  import BrushLab from './BrushLab.svelte';
  import { DrawingSession, type SessionState } from './session';
  import { ViewportPresentation, type PresentationState } from './presentation';
  import { ViewportInput } from './viewport-input';
  import { workspaceShortcut, type CameraMode, type Shortcut } from './camera-input';
  import { brushFor, isPaint, latestAppearances, restoredPanelWidth, sourceNames, title } from './drawing';
  import type { CompanionBootstrap, NprIntent, LookCatalogEntry, ModelDescriptor, NprSnapshot, NprStyleLayer, PlaybackCommand, PresentationMode } from './contracts';
  type Layer = NprStyleLayer;
  let sessionState = $state<SessionState>({snapshot:null,connection:'connecting',pending:false,error:''});
  let presentationState = $state<PresentationState>({mode:'jpeg',activeMode:null,size:[1,1],fps:0,error:'',nativeUnavailable:'Otwieranie hosta…',rgbaUnavailable:'Otwieranie hosta…'});
  let localError = $state(''), canPickFile = $state(false);
  let session: DrawingSession | undefined, presentation: ViewportPresentation | undefined, input: ViewportInput | undefined, host: CompanionHost | undefined;
  const snapshot=$derived(sessionState.snapshot), pending=$derived(sessionState.pending || sessionState.connection!=='connected');
  const error=$derived(localError || sessionState.error), status=$derived(sessionState.connection==='connected'?'Gotowe':'Otwieranie rysunku…');
  const connection=$derived(({connecting:'Łączenie',connected:'Połączono',resyncing:'Synchronizacja',disconnected:'Rozłączono'})[sessionState.connection]);
  let canvas = $state<HTMLCanvasElement>(undefined!), rgbaCanvas = $state<HTMLCanvasElement>(undefined!), viewport = $state<HTMLDivElement>(undefined!);
  const mode=$derived(presentationState.mode), activeMode=$derived(presentationState.activeMode), presentationError=$derived(presentationState.error), nativeUnavailable=$derived(presentationState.nativeUnavailable);
  const renderSize=$derived(presentationState.size), fps=$derived(presentationState.fps);
  let cameraMode = $state<CameraMode>('select'), selectedLayerId = $state<string | undefined>(), newSource = $state<Layer['source']>('silhouette');
  let showModels = $state(true), adding = $state<'line'|'paint'|undefined>(), leftWidth = $state(restoredPanelWidth('drawing-studio.left-width',240,200,360)), rightWidth = $state(restoredPanelWidth('drawing-studio.right-width',380,340,560));
  let draftDirty = $state(false), editorEpoch = $state(0), confirmation = $state<{ message:string; run:()=>void } | undefined>();
  let naming = $state<'look'|'variant'|undefined>(), documentName = $state('');
  const domain = $derived(snapshot?.values.npr as NprSnapshot | undefined), settings = $derived(domain?.settings), layers = $derived(settings?.style_layers.layers ?? []), selectedLayer = $derived(layers.find((layer) => layer.id === selectedLayerId));
  const selectedBrush = $derived(selectedLayer && selectedLayer.source !== 'paper' ? brushFor(selectedLayer, domain?.brushes ?? { brushes: {} }) ?? latestAppearances(domain?.brushes ?? { brushes: {} }, selectedLayer.source)[0] : undefined);
  const models = $derived(snapshot?.values.models as ModelDescriptor[] ?? []), visibleModels = $derived(models.filter((model) => model.state === 'ready')), selectedModel = $derived(visibleModels.find((model) => model.id === settings?.selected));
  const diagnostics = $derived(domain?.layer_diagnostics ?? {}), drafts = $derived(domain?.drafts ?? {}), variants = $derived(domain?.variants ?? {}), lookCatalog = $derived(snapshot?.values.look_catalog as Record<string, LookCatalogEntry> ?? {}), looks = $derived(Object.keys(lookCatalog).length ? Object.keys(lookCatalog) : domain?.available_looks ?? []);
  const activeLookPreview = $derived(domain?.active_look ? lookCatalog[domain.active_look]?.preview : undefined);
  const buildUp = $derived(domain?.build_up ?? 1), soloLayer = $derived(domain?.solo_layer ?? undefined), locked = $derived(domain?.locked_layers ?? []), selectedLocked = $derived(!!selectedLayer && locked.includes(selectedLayer.id));
  const documentStatus = $derived(error || (pending ? 'Przetwarzanie…' : draftDirty ? 'Podgląd niezapisanej edycji' : domain?.dirty ? 'Rysunek zmieniony' : status));
  const playback = $derived(snapshot?.values.playback), clips = $derived(snapshot?.values.animation_clips ?? []);
  $effect(() => { if (layers.length && !layers.some((layer) => layer.id === selectedLayerId)) selectedLayerId = layers.find((layer) => layer.source !== 'paper')?.id ?? layers[0].id; });
  $effect(() => { try { if (typeof localStorage !== 'undefined') { localStorage.setItem('drawing-studio.left-width', String(leftWidth)); localStorage.setItem('drawing-studio.right-width', String(rightWidth)); } } catch { /* Layout storage is optional; document state stays on the backend. */ } });
  function dispatch(control: string, intent: NprIntent) { if(session?.dispatch(control,intent)){localError='';return true;} localError='Połączenie nie jest gotowe. Poczekaj na synchronizację albo otwórz Drawing Studio ponownie.';return false; }
  function cancelPreview() { dispatch('layer.preview', {kind:'cancel_layer_preview'}); draftDirty = false; editorEpoch++; }
  function guard(run:()=>void, message = 'Porzucić niezastosowaną edycję pozycji?', preset = false) {
    if (draftDirty || (preset && domain?.look_dirty)) confirmation = {message,run};
    else run();
  }
  function confirmChange() { const run = confirmation?.run; confirmation = undefined; cancelPreview(); run?.(); }
  function selectLayer(id:string) { if (id !== selectedLayerId) guard(() => { selectedLayerId=id; editorEpoch++; }); }
  function selectPreset(id:string) { if (id && id !== domain?.active_look) guard(() => { cancelPreview(); dispatch('look.apply',{kind:'use_look',id}); },'Otworzyć wybrany preset i odrzucić niezapisane zmiany obecnego presetu?',true); }
  function restorePreset() { const id=domain?.active_look;if(id)guard(()=>{cancelPreview();dispatch('look.apply',{kind:'use_look',id});},'Przywrócić zapisany preset?',true); }
  function selectModel(model:ModelDescriptor) { if (model.id !== settings?.selected) guard(() => dispatch('source.select',{kind:'select_model',model:model.id}),'Otworzyć model i zachować bieżący preset?',true); }
  function addEntry() { guard(() => { dispatch('layer.add',{kind:'add_layer',source:newSource,label:sourceNames[newSource]}); adding=undefined; }); }
  function remove() { if (selectedLayer && selectedLayer.source !== 'paper') { const layer=selectedLayer.id; confirmation={message:`Usunąć „${selectedLayer.label}” z presetu?`,run:()=>dispatch('layer.delete',{kind:'delete_layer',layer})}; } }
  function duplicate() { if (selectedLayer) guard(() => dispatch('layer.duplicate',{kind:'duplicate_layer',layer:selectedLayer!.id})); }
  function move(direction:number) { if (selectedLayer) guard(() => dispatch('layer.move',{kind:'move_layer',layer:selectedLayer!.id,direction})); }
  function lock() { if (selectedLayer) guard(() => dispatch('layer.lock',{kind:'set_layer_lock',layer:selectedLayer!.id,locked:!selectedLocked})); }
  function toggleSolo() { dispatch('layer.solo',{kind:'set_solo_layer',layer:soloLayer===selectedLayerId?null:selectedLayerId??null}); }
  function setBuildUp(value:number) { dispatch('layer.build-up',{kind:'set_build_up',value}); }
  function coverage() { dispatch('layer.coverage',{kind:'set_debug_view',view:settings?.debug==='FeatureClasses'?'Final':'FeatureClasses'}); }
  function save(kind:'save_all'|'save_look') { if (!draftDirty) dispatch('save',{kind}); }
  function playbackCommand(command: PlaybackCommand) { dispatch('playback',{kind:'playback',command}); }
  function saveBrushVersion(brush: import('./contracts').BrushDefinition, updateMatching: boolean) {
    if (!selectedLayer || selectedLayer.source === 'paper' || selectedLocked || !domain) return;
    const versions = domain.brushes.brushes[brush.id] ?? [];
    const version = Math.max(0, ...versions.map((item) => item.version)) + 1;
    const next = { ...brush, version };
    dispatch('brush.save-and-pin', { kind: 'save_brush_version_and_pin', brush: next,
      layer: selectedLayer.id, update_matching: updateMatching });
  }
  function togglePlayback() { if (settings?.selected && playback) playbackCommand({kind:'playing',playing:!playback.playing}); }
  function openDraft(id:string) { guard(() => dispatch('draft.open',{kind:'open_draft',model:id}),'Otworzyć draft zamiast bieżącego rysunku?',true); }
  function applyVariant(id:string) { guard(() => dispatch('variant.apply',{kind:'apply_variant',id}),'Zastosować wariant do bieżącego rysunku?',true); }
  function saveNamed() { const id=documentName.trim(); if (!id || draftDirty) return; dispatch('save',{kind:naming==='look'?'save_as_look':'save_variant',id}); naming=undefined; }
  async function importModel() { try { const path = await host?.invoke<string|null>('pick_file',{filters:[{name:'Modele 3D',extensions:['glb','gltf']}]}); if(path) dispatch('source.import',{kind:'import_model',path}); } catch(reason) { localError=String(reason); } }
  function selectPresentation(value:string) { if(['native_gpu','jpeg','local_rgba'].includes(value)){input?.cancel();presentation?.select(value as PresentationMode);} }
  function shortcut(action: Shortcut | null) {
    if (action === 'save') { if (!draftDirty) save('save_all'); else localError = 'Najpierw zastosuj albo anuluj edycję pozycji.'; }
    else if (action === 'undo' || action === 'redo') { if (!draftDirty) dispatch('history', { kind: action }); }
    else if (action === 'cancel') { confirmation = undefined; cancelPreview(); input?.cancel(); }
    else if (action === 'play') togglePlayback();
    else if (action === 'orbit' || action === 'select') cameraMode = action;
  }
  function keydown(event: KeyboardEvent) {
    if (event.defaultPrevented) return;
    const target = event.target instanceof HTMLElement ? event.target : undefined;
    const editing = !!target && (['INPUT','SELECT','TEXTAREA'].includes(target.tagName) || target.isContentEditable);
    const interactive = !!target?.closest('button,a,summary,[role="button"]');
    const action = workspaceShortcut({key:event.key,ctrl:event.ctrlKey,shift:event.shiftKey,alt:event.altKey,meta:event.metaKey,repeat:event.repeat},editing,interactive);
    if (action) { event.preventDefault(); shortcut(action); }
  }
  onMount(() => {
    const boot = (window as unknown as { __PLAYGROUND__?: CompanionBootstrap }).__PLAYGROUND__;
    if (!boot) { sessionState={...sessionState,connection:'disconnected',error:'Otwórz Drawing Studio z aktywnej sceny Amigo.'}; return; }
    host=companionHost(boot);canPickFile=host.canPickFile;
    session=new DrawingSession({state:value=>sessionState=value,frame:blob=>{void presentation?.jpeg(blob);},
      connected:()=>presentation?.setConnected(true),disconnected:()=>{input?.cancel();presentation?.setConnected(false);}});
    presentation=new ViewportPresentation({viewport,jpeg:canvas,rgba:rgbaCanvas},host,{state:value=>presentationState=value,
      send:message=>session?.send(message)??false,input:event=>input?.native(event)});
    input=new ViewportInput(viewport,{enabled:()=>session?.ready??false,mode:()=>cameraMode,size:()=>renderSize,
      dispatch:intent=>dispatch('viewport',intent),interacting:active=>presentation?.request(active),shortcut});
    let disposed=false;
    void presentation.start().then(()=>{if(!disposed)session?.connect(boot);}).catch(reason=>{if(!disposed)sessionState={...sessionState,connection:'disconnected',error:String(reason)};});
    const blur=()=>input?.cancel();
    window.addEventListener('keydown',keydown);window.addEventListener('blur',blur);
    return ()=>{disposed=true;window.removeEventListener('keydown',keydown);window.removeEventListener('blur',blur);input?.dispose();presentation?.dispose();session?.dispose();};
  });
</script>

<div class="studio">
  <header class="topbar">
    <strong class="brand">NPR Drawing Studio</strong><span class="document">{selectedModel?.label ?? settings?.selected ?? 'Rysunek'}</span>
    <button class:playing={playback?.playing} disabled={!settings?.selected || pending || playback?.duration_seconds===0} onclick={togglePlayback}>{playback?.playing ? 'Ⅱ Pauza' : '▶ Play'}</button>
    <small title="Klip z pliku lub obrót modelu; kamera orbit jest niezależna">Ruch</small>
    <select aria-label="Źródło animacji" style="width:180px" disabled={!settings?.selected || pending} value={playback?.source.kind==='clip'?String(playback.source.index):'turntable'} onchange={(event) => playbackCommand({kind:'source',source:event.currentTarget.value==='turntable'?{kind:'turntable'}:{kind:'clip',index:Number(event.currentTarget.value)}})}><option value="turntable">Obrót modelu</option>{#each clips as clip}<option value={String(clip.index)}>{clip.name}</option>{/each}</select><i></i>
    <button disabled={!domain?.can_undo || pending || draftDirty} onclick={() => dispatch('history',{kind:'undo'})}>Cofnij</button>
    <button disabled={!domain?.can_redo || pending || draftDirty} onclick={() => dispatch('history',{kind:'redo'})}>Ponów</button>
    <button disabled={pending || draftDirty || !settings} onclick={() => save('save_all')}>Zapisz rysunek</button>
  </header>
  <main style={`grid-template-columns:minmax(160px,${leftWidth}px) 8px minmax(0,1fr) 8px minmax(280px,${rightWidth}px)`}>
    <aside class="left panel">
      <section><h2>Model <button onclick={() => showModels=!showModels}>{showModels?'Zwiń':'Rozwiń'}</button></h2><p>Jeden aktywny model. Wybór zachowuje preset.</p>
        {#if showModels}<div class="list">{#each visibleModels as model}<button class:active={model.id===settings?.selected} class="model" disabled={pending} onclick={() => selectModel(model)}>{#if model.thumbnail}<img src={model.thumbnail} alt="" />{/if}<span><strong>{model.label}</strong><small>{model.kind}</small></span></button>{/each}</div>{/if}
        <button class="wide" disabled={!settings || pending || !canPickFile} onclick={importModel}>Otwórz plik modelu…</button>
      </section>
      <section><h2>Drafty i warianty</h2><div class="list">
        {#each Object.keys(drafts) as id}<button disabled={pending} onclick={() => openDraft(id)}>Draft · {title(id)}</button>{/each}
        {#each Object.keys(variants) as id}<button disabled={pending} onclick={() => applyVariant(id)}>Wariant · {title(id)}</button>{/each}
      </div><button class="wide" disabled={!settings || draftDirty} onclick={() => { naming='variant';documentName=''; }}>Zapisz wariant…</button></section>
      <section><PlaybackPanel {playback} {clips} {pending} send={playbackCommand} /></section>
      <section><h2>Sterowanie</h2><p>Przeciągnij lewym przyciskiem: orbit. Środkowym: przesuwanie. Kółko: zoom.</p><p>O — orbit · V — zaznaczanie · Spacja — Play</p><p>Ctrl+Z — cofnij · Ctrl+S — zapisz rysunek</p></section>
    </aside>
    <PanelSplitter side="left" value={leftWidth} minimum={200} maximum={360} resize={(value) => leftWidth=value} />
    <ViewportPane bind:canvas bind:rgbaCanvas bind:viewport {activeMode} {cameraMode} {renderSize} {fps} ready={!!settings} message={presentationError} coverageActive={settings?.debug === 'FeatureClasses'} selectMode={() => cameraMode='select'} orbitMode={() => cameraMode='orbit'} toggleCoverage={coverage} pointerDown={event=>input?.pointerDown(event)} pointerMove={event=>input?.pointerMove(event)} pointerUp={event=>input?.pointerUp(event)} pointerCancel={()=>input?.cancel()} wheel={event=>input?.wheel(event)} />
    <PanelSplitter side="right" value={rightWidth} minimum={340} maximum={560} resize={(value) => rightWidth=value} />
    <aside class="right panel">
      <section class="preset-picker"><h2>Preset <small>{domain?.look_dirty?'Zmieniony':'Zapisany'}</small></h2>
        <label>Wybierz preset<select aria-label="Wybierz preset" value={domain?.active_look??''} disabled={pending} onchange={(event) => selectPreset(event.currentTarget.value)}>{#if !domain?.active_look}<option value="" disabled>Wybierz preset</option>{/if}{#each looks as id}<option value={id}>{title(id)}</option>{/each}</select></label>
        {#if domain?.active_look}<div class="preset-preview" style="display:grid;place-items:center;min-height:74px;margin:8px 0;border:1px solid #44483b;border-radius:5px;background:#131510">{#if activeLookPreview}<img src={activeLookPreview} alt={`Podgląd presetu: ${title(domain.active_look)}`} style="display:block;width:100%;height:74px;object-fit:contain" />{:else}<small>Przygotowywanie podglądu presetu…</small>{/if}</div>{/if}
        <div class="actions"><button disabled={!domain?.active_look || pending || draftDirty} onclick={() => save('save_look')}>Zapisz preset</button><button disabled={!settings || draftDirty} onclick={() => {naming='look';documentName='';}}>Zapisz jako…</button><button disabled={!domain?.active_look || pending} onclick={restorePreset}>Przywróć</button></div>
        {#if domain?.included_looks.length}<small>Dziedziczy ustawienia: {domain.included_looks.map(title).join(', ')}</small>{/if}
      </section>
      {#if confirmation}<section class="notice" role="alert"><p>{confirmation.message}</p><div class="actions"><button class="primary" onclick={confirmChange}>Potwierdź</button><button onclick={() => confirmation=undefined}>Anuluj</button></div></section>{/if}
      {#if naming}<section><form onsubmit={(event) => {event.preventDefault();saveNamed();}}><label>{naming==='look'?'Nazwa nowego presetu':'Nazwa wariantu'}<input aria-label="Nazwa dokumentu" bind:value={documentName} pattern="[a-zA-Z0-9_-]+" maxlength="80" required placeholder="moj-rysunek" /></label><small>Litery bez spacji, cyfry, myślnik lub podkreślenie.</small><div class="actions"><button type="submit" disabled={pending || draftDirty}>Zapisz</button><button type="button" onclick={() => naming=undefined}>Anuluj</button></div></form></section>{/if}
      {#if domain}
        <PresetTree {layers} library={domain.brushes} previews={snapshot?.values.layer_previews??{}} brushPreviews={snapshot?.values.brush_previews??{}} {locked} {selectedLayerId} {selectLayer} disabled={pending || draftDirty} toggle={(layer) => dispatch('layer.enabled',{kind:'set_layer_enabled',layer:layer.id,enabled:!layer.enabled})} />
        <section><div class="actions"><button disabled={pending} onclick={() => {adding='line';newSource='silhouette';}}>＋ Linia</button><button disabled={pending} onclick={() => {adding='paint';newSource='wash';}}>＋ Farba</button></div>
          {#if adding}<label>{adding==='line'?'Rodzaj linii':'Rodzaj farby'}<select bind:value={newSource}>{#each Object.entries(sourceNames).filter(([id]) => id!=='paper' && isPaint(id as Layer['source'])===(adding==='paint')) as [id,name]}<option value={id}>{name}</option>{/each}</select></label><div class="actions"><button disabled={pending} onclick={addEntry}>Dodaj</button><button onclick={() => adding=undefined}>Anuluj</button></div>{/if}
          {#if selectedLayer}<div class="actions"><button class:active={soloLayer===selectedLayerId} onclick={toggleSolo}>Solo</button><button onclick={lock}>{selectedLocked?'Odblokuj':'Zablokuj'}</button><button disabled={selectedLayer.source==='paper' || selectedLocked || pending} onclick={duplicate}>Powiel</button><button disabled={selectedLayer.source==='paper' || selectedLocked || pending} onclick={remove}>Usuń</button></div>
            <details><summary>Kolejność nakładania</summary><p>Grupy powyżej nie zmieniają kolejności renderowania.</p><ol>{#each layers as item}<li class:current={item.id===selectedLayerId}>{item.label}</li>{/each}</ol><div class="actions"><button disabled={selectedLayer.source==='paper' || selectedLocked || pending || layers.findIndex((l)=>l.id===selectedLayerId)<=1} onclick={() => move(-1)}>↑ Wcześniej</button><button disabled={selectedLayer.source==='paper' || selectedLocked || pending || layers.at(-1)?.id===selectedLayerId} onclick={() => move(1)}>↓ Później</button></div></details>
          {/if}
        </section>
        {#if selectedLayer && settings}
          {#key selectedLayer.id+':'+editorEpoch}<EntryInspector layer={selectedLayer} library={domain.brushes} brushPreviews={snapshot?.values.brush_previews??{}} style={settings.global} model={settings.selected} locked={selectedLocked} {pending} image={snapshot?.values.layer_previews?.[selectedLayer.id]} imageError={snapshot?.values.layer_preview_errors?.[selectedLayer.id]} onDirty={(value)=>draftDirty=value} cancel={cancelPreview} previewChange={(value)=>dispatch('layer.preview',{kind:'preview_layer',layer:value.id,value})} apply={(value)=>dispatch('layer.edit',{kind:'replace_layer',layer:value.id,value})} saveAppearance={(value,name,update_matching)=>dispatch('brush.save',{kind:'save_appearance',layer:value.id,value,name,update_matching})} />{/key}
          {#if selectedBrush}<BrushLab brush={selectedBrush} layer={selectedLayer} preview={snapshot?.values.brush_previews?.[`${selectedBrush.id}@${selectedBrush.version}`] ?? undefined} {pending} disabled={draftDirty || selectedLocked} save={saveBrushVersion} />{/if}
          <section><h2>Diagnostyka pozycji</h2><p title="Szacowane pokrycie próbek geometrii, nie licznik pikseli GPU">{diagnostics[selectedLayer.id]?.no_effect_reason??'Aktywna'} · ślady {diagnostics[selectedLayer.id]?.generated_marks??0} · pokrycie ≈{Math.round((diagnostics[selectedLayer.id]?.mask_coverage??0)*100)}%</p><dl><dt>Geometria źródłowa</dt><dd>{diagnostics[selectedLayer.id]?.source_geometry??0}</dd><dt>Trójkąty śladu</dt><dd>{diagnostics[selectedLayer.id]?.generated_triangles??0}</dd><dt>Ekstrakcja</dt><dd>{diagnostics[selectedLayer.id]?.extraction_micros??0} μs</dd><dt>Tesselacja</dt><dd>{diagnostics[selectedLayer.id]?.tessellation_micros??0} μs</dd></dl></section>
        {/if}
      {/if}
      <section><details><summary>Renderowanie</summary><dl><dt>FPS</dt><dd>{fps}</dd><dt>Rozmiar</dt><dd>{renderSize[0]} × {renderSize[1]}</dd><dt>Prezentacja</dt><dd>{activeMode??mode}</dd></dl><label>Tryb prezentacji<select value={mode} onchange={(event)=>selectPresentation(event.currentTarget.value)}><option value="native_gpu" disabled={!!nativeUnavailable}>Native GPU</option><option value="jpeg">JPEG</option><option value="local_rgba" disabled={!!presentationState.rgbaUnavailable}>Local RGBA</option></select></label><label>Budowanie rysunku <output>{Math.round(buildUp*100)}%</output><input type="range" min="0" max="1" step=".05" value={buildUp} disabled={pending} oninput={(event)=>setBuildUp(Number(event.currentTarget.value))}/></label></details></section>
    </aside>
  </main>
  <footer class="statusbar"><span class:problem={!!error} role="status">{connection} · {documentStatus}</span><i></i><small>Separatory paneli: przeciągnij lub użyj klawiszy strzałek</small></footer>
</div>

<style>
  :global(*){box-sizing:border-box}:global(body){margin:0;background:#111315;color:#e8e5df;font:13px/1.4 Inter,"Segoe UI",sans-serif}:global(button),:global(input),:global(select){font:inherit}:global(button){border:1px solid #41413e;border-radius:5px;background:#272826;color:inherit;padding:6px 8px;cursor:pointer}:global(button:hover:not(:disabled)){background:#393a36}:global(button:disabled){opacity:.45;cursor:default}:global(button:focus-visible),:global(input:focus-visible),:global(select:focus-visible),:global(summary:focus-visible){outline:2px solid #d3aa65;outline-offset:2px}.studio{height:100dvh;display:grid;grid-template-rows:50px minmax(0,1fr) 30px;overflow:hidden}.topbar{display:flex;align-items:center;gap:9px;padding:0 14px;border-bottom:1px solid #363632;background:#20211e}.brand{color:#fbf6eb;font-size:13px}.document{color:#d3aa65}.topbar small{font-size:10px;color:#949784}.playing,.primary{border-color:#b58d4c;background:#51432a}.problem{color:#ee9a87!important}i{flex:1}main{display:grid;min-height:0}.panel{min-height:0;overflow:auto;background:#20211e}.left{border-right:1px solid #363632}.right{border-left:1px solid #363632}.panel>section{padding:12px;border-bottom:1px solid #363632}h2{display:flex;align-items:center;justify-content:space-between;gap:6px;margin:0 0 9px;color:#f7f1e4;font-size:13px}small,p{color:#a8a79f;font-size:11px}p{margin:0 0 9px}.list{display:grid;gap:6px}.model{display:flex;align-items:center;gap:8px;width:100%;text-align:left}.active,.model.active{border-color:#c49a54;background:#38342a}.model img{width:40px;height:40px;object-fit:contain;background:#171816}.model strong,.model small{display:block}.actions{display:flex;flex-wrap:wrap;gap:6px;margin:9px 0}.actions button{font-size:11px}.wide{width:100%;margin-top:10px;font-size:11px}label{display:grid;gap:4px;margin:8px 0;color:#c8c6be;font-size:11px}input,select{width:100%;accent-color:#c69b55}input:not([type=range]),select{border:1px solid #484942;border-radius:4px;background:#151613;color:#e8e5df;padding:6px}output{float:right;color:#d3aa65}.notice{background:#453923!important;border:1px solid #a77e3c}.notice p{color:#f2d6a1}details{margin-top:8px}summary{cursor:pointer;color:#bbbba9;font-size:11px}details p{margin-top:8px}ol{padding-left:22px;color:#8d9485;font-size:11px}.current{color:#e9cc93}dl{display:grid;grid-template-columns:1fr 1fr;gap:4px;font-size:11px}dd{margin:0;text-align:right;color:#d3aa65}.statusbar{display:flex;align-items:center;gap:12px;padding:0 12px;border-top:1px solid #363632;background:#20211e;color:#9b9b91;font-size:11px}.statusbar small{color:#8f9387}@media(max-width:1050px){.topbar small{display:none}}@media(max-width:760px){.studio{height:auto;min-height:100dvh;overflow:visible;grid-template-rows:auto 1fr auto}.topbar{flex-wrap:wrap;padding:10px}.topbar i{display:none}main{grid-template-columns:1fr!important;grid-template-rows:auto minmax(55dvh,1fr) auto}.left{max-height:220px}.right{max-height:none}.statusbar{flex-wrap:wrap;padding:9px}}
</style>
