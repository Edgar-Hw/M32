# M32 0.0.3-T010 FIX1 — Evidence Filename Convention

## Trigger

The first T010 version-close run failed with:

```text
Missing required First Frame task evidence:
docs/spec/task-evidence/0.0.3-T001.md
```

The evidence was not missing.

The repository already contained the complete T001-T009 evidence set under the established naming
convention:

```text
M32_0.0.3-T001_evidence.md
...
M32_0.0.3-T009_evidence.md
```

This convention is also consistent with earlier M32 task evidence.

## Root Cause

The initial T010 verifier incorrectly assumed a new filename format:

```text
0.0.3-T001.md
```

instead of the repository's actual format:

```text
M32_0.0.3-T001_evidence.md
```

## FIX1

The version-close evidence loop now resolves each task as:

```text
M32_<TaskId>_evidence.md
```

Example:

```text
TaskId       = 0.0.3-T001
EvidenceFile = M32_0.0.3-T001_evidence.md
```

No evidence files need to be renamed or duplicated.

## Scope

Verifier/documentation only.

No Rust source change.

No runtime behavior change.

No dependency/Cargo.lock change.

No WIE/RustJava change.

## Status

```text
0.0.3-T010 = FIX1 / IN_PROGRESS
```
