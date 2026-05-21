# Codemap commands

Build a stable codemap binary:

```powershell
cargo build -p amigo-codemap
Copy-Item target\debugmigo-codemap.exe target\debugmigo-codemap-stable.exe
$cm = "target\debugmigo-codemap-stable.exe"
```

Common commands:

```powershell
& $cm brief
& $cm changes --compact --hide-generated --limit 20
& $cm change-plan "<task>" --limit 20
& $cm open-set "<symbols paths topic>" --why --limit 20
```

Use codemap first. Use `rg` second. Open files last.
