import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';
import PresetTree from './PresetTree.svelte';
import { appearanceLabel, brushFor, category, latestAppearances, restoredPanelWidth, useAppearance } from './drawing';
import { MutationQueue } from './mutation-queue';
import type { BrushDefinition, BrushLibrary, NprStyleLayer } from './contracts';

const brush: BrushDefinition = { id:'pen',name:'Cienkopis',version:1,medium:'ink',applications:['stroke'],width:1,taper:.2,softness:.1,spacing:.2,pressure_profile:.7,irregularity:.1,correction:.2,dryness:.2,seed:1 };
const library: BrushLibrary = { brushes: { pen:[brush,{...brush,version:2,width:3}],paint:[{...brush,id:'paint',name:'Akwarela',applications:['surface'],medium:'watercolour-wash'}] } };
const line: NprStyleLayer = { id:'outline',label:'Kontur',source:'silhouette',enabled:true,opacity:1,blend:'normal',color_source:{kind:'style-palette'},brush:{brush:{id:'pen',version:1},width:2},target:{kind:'all'},mask:{kind:'none'} };

describe('drawing preset presentation', () => {
  it('restores layout before saving and bounds invalid storage values', () => {
    expect(restoredPanelWidth('panel',380,340,560,{getItem:()=> '440'})).toBe(440);
    expect(restoredPanelWidth('panel',380,340,560,{getItem:()=> '900'})).toBe(560);
    expect(restoredPanelWidth('panel',380,340,560,{getItem:()=> 'NaN'})).toBe(380);
    expect(restoredPanelWidth('panel',380,340,560,{getItem:()=> null})).toBe(380);
  });
  it('groups the existing ordered stack without changing document order', () => {
    const sources = ['silhouette','wash','creases','paper'] as const;
    expect(sources.map(category)).toEqual(['Linie','Farby','Linie','Papier']);
    expect(sources).toEqual(['silhouette','wash','creases','paper']);
  });
  it('shows one compatible template per identity but resolves pinned older appearances', () => {
    expect(latestAppearances(library,'silhouette').map((item)=>item.version)).toEqual([2]);
    expect(latestAppearances(library,'wash').map((item)=>item.id)).toEqual(['paint']);
    expect(latestAppearances(library,'paper')).toEqual([]);
    expect(brushFor(line,library)?.version).toBe(1);
    expect(library.brushes.pen[0].version).toBe(1);
  });
  it('applies appearance to one entry without changing its geometry or another entry', () => {
    const other = structuredClone(line);
    const updated = useAppearance(line,{...brush,id:'pencil',medium:'graphite'});
    expect(updated.source).toBe(line.source);
    expect(updated.target).toEqual(line.target);
    expect(updated.brush?.width).toBeUndefined();
    expect(other).toEqual(line);
    expect(appearanceLabel(updated,{brushes:{pencil:[{...brush,id:'pencil',medium:'graphite'}]}})).toBe('Ołówek');
  });
  it('renders spaced groups, real image slots and no technical versions or brush child rows', () => {
    const {body} = render(PresetTree,{props:{layers:[line,{...line,id:'paper',label:'Papier',source:'paper',brush:undefined}],library,previews:{outline:'data:image/svg+xml,sample'},selectedLayerId:'outline',selectLayer:()=>{},toggle:()=>{}}});
    expect(body).toContain('Zawartość presetu');
    expect(body).toContain('Linie');
    expect(body).toContain('Farby');
    expect(body).toContain('Papier');
    expect(body).toContain('aria-expanded="true"');
    expect(body).toContain('aria-pressed="true"');
    expect(body).toContain('data:image/svg+xml,sample');
    expect(body).not.toContain('pen@1');
    expect(body).not.toContain('v1');
    expect(body).not.toContain('Detach');
    expect(body).not.toContain('Widoczność: Papier');
  });
  it('coalesces preview drafts without dropping Apply or Cancel boundaries', () => {
    const queue = new MutationQueue();
    for (const opacity of [.1,.2,.7]) queue.push('preview',{kind:'preview_layer',layer:'outline',value:{...line,opacity}});
    queue.push('edit',{kind:'replace_layer',layer:'outline',value:{...line,opacity:.7}});
    const preview = queue.take(1)!;
    expect(preview.intent).toMatchObject({kind:'preview_layer',value:{...line,opacity:.7}});
    queue.accepted(preview.request_id,2,2);
    const apply = queue.take(2)!;
    expect(apply.intent.kind).toBe('replace_layer');
    queue.push('preview',{kind:'cancel_layer_preview'});
    queue.accepted(apply.request_id,3,3);
    expect(queue.take(3)?.intent.kind).toBe('cancel_layer_preview');
  });
});
