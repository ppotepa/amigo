export type Snapshot = { revision: number; values: Record<string, any>; metadata: unknown };
export type Delta = { base_revision: number; revision: number; changed: Record<string, any>; removed: string[]; metadata?: unknown };
export function applyDelta(snapshot: Snapshot, delta: Delta): Snapshot {
  if (snapshot.revision !== delta.base_revision) throw new Error('State revision mismatch; resynchronizing session.');
  const values = { ...snapshot.values, ...delta.changed };
  for (const key of delta.removed) delete values[key];
  return { revision: delta.revision, values, metadata: delta.metadata ?? snapshot.metadata };
}

export type PresentationMode = 'native_gpu' | 'local_rgba' | 'jpeg';
export type FrameHeader = { sequence: number; revision: number; generation: number; size: [number, number]; format: 'rgba8_srgb'; mode: PresentationMode; input_id: number; stages: Record<string, number> };
export type EncodedFrame = { header: FrameHeader; image: Blob };
export type FrameAck = { sequence: number; generation: number; presented: boolean; decode_ms: number; present_ms: number };

export async function parseFrame(blob: Blob): Promise<EncodedFrame> {
  const prefix = new DataView(await blob.slice(0, 8).arrayBuffer());
  if (prefix.byteLength !== 8 || prefix.getUint32(0) !== 0x41505646) throw new Error('Invalid viewport frame');
  const length = prefix.getUint32(4, true);
  if (!length || length > 16384 || length + 8 >= blob.size) throw new Error('Invalid viewport header length');
  const header = JSON.parse(await blob.slice(8, 8 + length).text()) as FrameHeader;
  if (header.mode !== 'jpeg' || header.format !== 'rgba8_srgb' || header.size.some(n => !Number.isInteger(n) || n < 1 || n > 16384)) throw new Error('Unsupported viewport frame');
  return { header, image: blob.slice(8 + length, undefined, 'image/jpeg') };
}

/** One decoder, one newest waiting image and one newest ready bitmap. */
export class LatestFrameDecoder {
  private waiting: EncodedFrame | null = null;
  private ready: { image: ImageBitmap; header: FrameHeader; decode: number } | null = null;
  private animation: number | undefined;
  private decoding = false;
  private disposed = false;
  private newestSequence = -1;
  constructor(private draw: (image: ImageBitmap, header: FrameHeader) => void, private error: (message: string) => void,
    private acknowledge: (ack: FrameAck) => void, private current: (header: FrameHeader) => boolean) {}
  private ack(header: FrameHeader, presented = false, decode_ms = 0, present_ms = 0) {
    this.acknowledge({ sequence: header.sequence, generation: header.generation, presented, decode_ms, present_ms });
  }
  push(frame: EncodedFrame) {
    if (this.disposed || !this.current(frame.header) || frame.header.sequence <= this.newestSequence) { this.ack(frame.header); return; }
    this.newestSequence = frame.header.sequence;
    if (this.waiting) this.ack(this.waiting.header);
    this.waiting = frame; void this.pump();
  }
  dispose() {
    this.disposed = true;
    if (this.animation !== undefined) cancelAnimationFrame(this.animation);
    if (this.waiting) this.ack(this.waiting.header);
    if (this.ready) { this.ready.image.close(); this.ack(this.ready.header); }
    this.waiting = null; this.ready = null;
  }
  private present = () => {
    this.animation = undefined;
    const ready = this.ready; this.ready = null;
    if (!ready) return;
    const start = performance.now();
    let presented = false;
    try {
      if (!this.disposed && this.current(ready.header)) { this.draw(ready.image, ready.header); presented = true; }
    } catch (error) { this.error(String(error)); }
    finally { ready.image.close(); this.ack(ready.header, presented, ready.decode, performance.now() - start); }
  };
  private async pump() {
    if (this.decoding) return;
    this.decoding = true;
    while (this.waiting && !this.disposed) {
      const frame = this.waiting; this.waiting = null;
      const start = performance.now();
      try {
        const image = await createImageBitmap(frame.image);
        if (image.width !== frame.header.size[0] || image.height !== frame.header.size[1]) {
          image.close(); throw new Error('Viewport frame dimensions disagree with header');
        }
        if (this.disposed || !this.current(frame.header)) { image.close(); this.ack(frame.header); continue; }
        if (this.ready) { this.ready.image.close(); this.ack(this.ready.header); }
        this.ready = { image, header: frame.header, decode: performance.now() - start };
        if (this.animation === undefined) this.animation = requestAnimationFrame(this.present);
      } catch (error) { this.ack(frame.header); if(!this.disposed && this.current(frame.header))this.error(String(error)); }
    }
    this.decoding = false;
  }
}
