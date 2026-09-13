<script lang="ts">
  import type { AnimationClip, AnimationTrack, ModelPlayback, PlaybackCommand } from './contracts';
  let { playback, clips, pending, send }: {
    playback: ModelPlayback | undefined; clips: AnimationClip[]; pending: boolean;
    send: (command: PlaybackCommand) => void;
  } = $props();
  const properties: Record<AnimationTrack['property'], string> = {
    translation: 'Pozycja', rotation: 'Obrót', scale: 'Skala', weights: 'Kształt (morph)',
  };
  const selectedIndex = $derived(playback?.source.kind === 'clip' ? playback.source.index : undefined);
  const selected = $derived(clips.find(c => c.index === selectedIndex));
  let scrubbing = $state<number | undefined>();
</script>

<h2>Animacja</h2>
{#if playback}
  <p>{selected?.name ?? 'Obrót modelu'} · <output aria-label="Czas animacji">{playback.time_seconds.toFixed(2)} s</output></p>
  {#if playback.duration_seconds !== null}
    <label>Pozycja w klipie
      <input aria-label="Pozycja w klipie" type="range" min="0" max={playback.duration_seconds} step="0.001"
        value={scrubbing ?? playback.time_seconds} disabled={pending || playback.duration_seconds === 0}
        oninput={(event) => scrubbing=Number(event.currentTarget.value)}
        onblur={() => scrubbing=undefined}
        onchange={(event) => { send({kind:'seek',seconds:Number(event.currentTarget.value)}); scrubbing=undefined; }} />
    </label>
    <p>Długość: {playback.duration_seconds.toFixed(2)} s</p>
  {/if}
  <div class="transport">
    <button disabled={pending} onclick={() => send({kind:'seek',seconds:0})}>Do początku</button>
    <label>Tempo<select aria-label="Tempo animacji" value={playback.speed} disabled={pending}
      onchange={(event) => send({kind:'options',looping:playback!.looping,speed:Number(event.currentTarget.value)})}>
      {#each [0.25,0.5,1,1.5,2,4] as speed}<option value={speed}>{speed}×</option>{/each}
    </select></label>
  </div>
  {#if playback.duration_seconds !== null}
    <label class="loop"><input type="checkbox" checked={playback.looping} disabled={pending}
      onchange={(event) => send({kind:'options',looping:event.currentTarget.checked,speed:playback!.speed})} />Powtarzaj klip</label>
  {/if}
{/if}
{#if clips.length}
  <details open><summary>Ścieżki z pliku · {clips.length} klipów</summary>
    <ul class="clips">
      {#each clips as clip (clip.index)}
        <li><details open={selected?.index === clip.index}><summary>{clip.name} · {clip.duration_seconds.toFixed(2)} s</summary>
          <ul class="tracks">
            {#each clip.tracks as track}
              <li><strong>{track.node_path}</strong><span>{properties[track.property]}</span><small>{track.keyframes} klatek · {track.interpolation}</small></li>
            {/each}
          </ul>
        </details></li>
      {/each}
    </ul>
  </details>
{:else}
  <p>Ten model nie zawiera klipów animacji. Play uruchamia obrót modelu.</p>
{/if}
<p>Kamera orbit działa niezależnie. Odtwarzanie nie zmienia presetu ani historii rysunku.</p>

<style>
  h2{margin:0 0 9px;font-size:13px}p,small{color:#a8a79f;font-size:11px}p{margin:7px 0}
  label{display:grid;gap:5px;font-size:11px;color:#c8c6be}input,select{accent-color:#c69b55}
  input[type=range]{width:100%}select{background:#151613;color:inherit;padding:4px;border:1px solid #484942;border-radius:4px}
  .transport{display:flex;gap:8px;align-items:end;margin:9px 0}.transport button{font-size:11px}.loop{display:flex;align-items:center;margin:9px 0}
  summary{cursor:pointer;font-size:11px;color:#dbccad;overflow-wrap:anywhere}details{margin:9px 0}
  ul{list-style:none;margin:8px 0;padding-left:12px}.tracks{border-left:1px solid #555044;padding-left:14px}
  .tracks li{display:grid;gap:3px;margin:12px 0;font-size:11px;overflow-wrap:anywhere}.tracks strong{font-weight:500}
  .tracks span{color:#d3aa65}.clips{max-height:280px;overflow:auto;padding-right:4px}
</style>
