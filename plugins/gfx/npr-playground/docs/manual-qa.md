# Drawing Studio manual QA

Run `drawing-studio` with the `playground` profile. These checks require the
actual window; a successful unit-test run does not constitute visual acceptance.

- [ ] Initial model is static. Top-bar Play starts/stops rotation independently
  of camera orbit.
- [ ] Import animated glTF and GLB. Select each file clip beside Play; verify
  named node paths, properties, keyframes and interpolation in the Animation panel.
  Exercise transform, morph and skinned clips. Seek, change speed, loop, pause,
  resume and restart after a non-looping clip ends. The playhead must update
  without causing camera-action revision conflicts or changing dirty/undo state.
  Camera movement and live entry edits preserve playback; switching models resets
  it and preserves the preset. A model without clips offers only its turntable.
- [ ] Primary drag and right drag orbit; middle drag pans; wheel zooms. Verify
  Native GPU, JPEG and Local RGBA, including resize and mode changes.
  Separate wheel bursts must produce separate undo gestures. Losing pointer
  capture ends dragging. Local RGBA must occupy the same rectangle as JPEG.
- [ ] The right picker opens a preset directly. Switching a modified preset
  offers explicit discard/cancel and never edits the previously selected preset.
- [ ] Only one model is visible. Selecting another model preserves the preset.
  Reopen a Draft and verify its model, camera and drawing state.
- [ ] Lines/Paints/Paper expand/collapse. Entries have real samples and spacing.
  Each entry exposes its assigned brush as an indented preview row. The preset
  picker also shows the backend-generated preset preview; no revision suffixes
  or decorative CSS swatches are shown.
- [ ] Add two hatch entries with different angles/spacing and a wash. All coexist;
  editing either hatch leaves the other unchanged.
- [ ] Change width, tool, pressure and correction in the inline inspector.
  The actual viewport previews the draft. Cancel/Escape restores it; Apply is
  one undo step. Disconnecting during preview discards the draft.
- [ ] Verify paint wash/granulation in the actual viewport, not just the SVG
  sample. Compare target, colour, opacity, blend and compositing order.
- [ ] Add/remove nested masks. Height follows object-local Y, tone follows
  surface illumination, and direction follows the actual surface normal.
  A narrow height band must cross a face even when all its corners are outside
  the range. Camera orbit must not move a local mask. Noise seed and inversion
  must affect both standalone and nested masks.
- [ ] Hide an entry before rendering, then show it: geometry must return.
  Change opacity to zero and verify diagnostics update without rebuilding paths.
- [ ] Duplicate, delete, lock and reorder entries. Paper remains pinned.
  Delete an inherited entry, save, restart and verify it stays deleted.
- [ ] Save an appearance to one entry; explicitly update all matching uses.
  A locked affected entry rejects the whole operation. Undo restores references
  and library together; old pinned appearances remain reproducible.
- [ ] Open Brush Lab from a non-paper entry. Edit medium, application, tool and
  stroke parameters, save a new version, verify the current entry is pinned to
  it, and exercise the optional update of all matching entries. Removing a
  brush assignment leaves Paper protected and keeps the layer editable.
- [ ] Save Preset and Save As with a custom appearance, then reopen from a fresh
  session. The preset carries its definitions and preserves includes/removals.
- [ ] Camera changes do not mark the preset modified. Save Drawing, Drafts and
  Variants retain their respective document scope.
- [ ] Tab, arrows, Home/End, Enter/Space work in the grouped list. Ctrl+Enter
  applies an entry. Ctrl+Z/Shift+Z and Ctrl+S work outside text editing.
  In the native viewport, Space toggles Play once per key press; O/V change
  tools and Escape cancels. Space on a focused button activates that button.
- [ ] Resize both panels and restart; widths restore without overwriting saved
  values during startup. Check narrow-window layout and native viewport bounds.
