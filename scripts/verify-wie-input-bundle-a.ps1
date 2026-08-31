$ErrorActionPreference = "Stop"

Write-Host "M32 0.0.4 Input Bundle A Verification"
Write-Host "====================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-integration.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Previous First Frame integration chain failed with exit code $LASTEXITCODE"
}

cargo test -p m32-emulator-api tests::m32_key_contract_covers_complete_pinned_feature_phone_surface -- --exact
if ($LASTEXITCODE -ne 0) { throw "T001 M32Key contract test failed" }

cargo test -p m32-emulator-api tests::guest_input_event_preserves_key_and_phase -- --exact
if ($LASTEXITCODE -ne 0) { throw "T001 GuestInputEvent phase test failed" }

cargo test -p m32-emulator-api tests::session_trait_accepts_backend_agnostic_input_events -- --exact
if ($LASTEXITCODE -ne 0) { throw "T002 EmulatorSession input seam test failed" }

cargo test -p m32-wie-adapter tests::m32_key_maps_exactly_to_pinned_wie_key_code -- --exact
if ($LASTEXITCODE -ne 0) { throw "T003 exact WIE key mapping test failed" }

cargo test -p m32-wie-adapter tests::m32_input_event_maps_press_release_repeat_to_wie_event -- --exact
if ($LASTEXITCODE -ne 0) { throw "T003 WIE event phase mapping test failed" }

cargo test -p m32-wie-adapter tests::wie_session_forwards_m32_input_events_to_pinned_backend -- --exact
if ($LASTEXITCODE -ne 0) { throw "T003 WieSession input forwarding test failed" }

Write-Host ""
Write-Host "[PASS] T001 exact 24-key M32Key surface"
Write-Host "[PASS] T001 backend-agnostic KeyDown/KeyUp/KeyRepeat contract"
Write-Host "[PASS] T002 EmulatorSession input dispatch seam"
Write-Host "[PASS] T003 exact M32Key -> pinned WIE KeyCode mapping"
Write-Host "[PASS] T003 KeyDown -> WIE Keydown"
Write-Host "[PASS] T003 KeyUp -> WIE Keyup"
Write-Host "[PASS] T003 KeyRepeat -> WIE Keyrepeat"
Write-Host "[PASS] T003 WieSession forwards input into pinned backend"
Write-Host "[PASS] previous First Frame integration chain"
Write-Host "M32 0.0.4 Input Bundle A verification passed."
exit 0
