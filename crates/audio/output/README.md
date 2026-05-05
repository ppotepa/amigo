# amigo-audio-output

Platform audio output backend built on CPAL.

## Responsibility
- Start and manage system audio output.
- Stream mixed samples to the device.
- Report backend/device diagnostics.

## Not here
- Audio mixing policy.
- Generated audio parsing.
- Scene audio commands.

## Depends on
- amigo-audio-mixer.
- amigo-core.
- amigo-capabilities.
- amigo-runtime.
- cpal.
