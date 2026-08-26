# Repository scripts

The codemap workflow is available without platform-specific binary-copy steps.

Linux/macOS:

```sh
./scripts/codemap.sh brief
./scripts/codemap.sh change-plan "describe the task" --limit 20
./scripts/codemap.sh open-set "topic or symbol" --why --limit 20
```

Windows PowerShell:

```powershell
./scripts/codemap.ps1 brief
./scripts/codemap.ps1 change-plan "describe the task" --limit 20
./scripts/codemap.ps1 open-set "topic or symbol" --why --limit 20
```

Both wrappers run the same `amigo-codemap` Cargo package and forward all arguments unchanged. Direct `cargo run -p amigo-codemap -- <command>` is also supported on every platform.
