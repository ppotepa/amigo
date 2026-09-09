# Panel API

Backend-independent scene panel documents, property snapshots and bounded framed
IPC messages. Reuses scene UI layout nodes and RuntimeControlService value types.
No egui, window backend or playground dependency. See
`docs/architecture/runtime-panels.md` at the repository root.

`PanelSnapshot` is the complete runtime view consumed by any host. Its
generation and revision identify the document/value epoch that a host must echo
when submitting an interaction through the panel service.
