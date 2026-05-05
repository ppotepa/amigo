# amigo-audio-mixer

Audio mixdown service for active sources.

## Responsibility
- Mix queued one-shots and realtime generated sources.
- Produce audio frames.
- Bridge playback state to platform output.

## Not here
- Device streaming.
- Scene command model.
- Filesystem asset loading.

## Depends on
- amigo-assets.
- amigo-audio-api.
- amigo-audio-generated.
- amigo-capabilities.
- amigo-core.
- amigo-runtime.
