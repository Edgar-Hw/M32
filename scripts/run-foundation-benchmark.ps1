$ErrorActionPreference = "Stop"

Write-Host "M32 Foundation Baseline Benchmark"
Write-Host "================================"
rustc --version
cargo --version
Write-Host ""

cargo bench -p m32-test-fixtures --bench foundation_baseline
exit $LASTEXITCODE
