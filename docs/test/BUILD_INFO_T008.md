# M32 0.0.1-T008 — BuildInfo

이 Task는 M32 binary가 자신의 build provenance를 구조화해 읽을 수 있게 만든다.

## 고정 필드

- `app_version`
- `product_spec_version`
- `spec_bundle_version`
- `git_commit`
- `wie_commit`
- `rust_version`
- `target`
- `build_profile`

wall-clock build timestamp는 재현성을 위해 포함하지 않는다.

`git_commit`은 build 시점의 repository `HEAD`를 기록한다. Working tree의 dirty 상태를
T008에서 별도 public field로 추가하지 않는다.

## 확인

```powershell
cargo run -p m32-desktop
```

출력의 `git_commit`은 다음과 비교한다.

```powershell
git rev-parse HEAD
```
