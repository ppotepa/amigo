$ErrorActionPreference = "Stop"

cargo run --quiet -p amigo-codemap -- @Args
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
