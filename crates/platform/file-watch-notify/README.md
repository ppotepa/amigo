# amigo-file-watch-notify

Notify-backed file watching service.

## Responsibility
- Bridge `notify` filesystem events into amigo-file-watch-api.
- Register file watch runtime service.
- Support hot-reload infrastructure.

## Not here
- Hot-reload orchestration.
- Asset parsing.
- Scene rehydration.

## Depends on
- amigo-core.
- amigo-file-watch-api.
- amigo-runtime.
- notify.
