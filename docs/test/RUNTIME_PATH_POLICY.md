# M32 Runtime Path Policy

Status: LOCKED BASELINE
Task: `0.0.1-T011`

## 1. Windows v1.0 root

M32 v1.0 Windows runtime data root:

```text
%LOCALAPPDATA%\M32
```

M32 does not use the repository directory, executable directory, current working directory,
Documents, Desktop, or a temporary directory as its normal persistent data root.

## 2. Stable paths

```text
%LOCALAPPDATA%\M32\
├─ config.json
├─ cache\
├─ logs\
└─ crashes\
```

Stable contract:

- root: `%LOCALAPPDATA%\M32`
- config: `%LOCALAPPDATA%\M32\config.json`
- cache: `%LOCALAPPDATA%\M32\cache`
- logs: `%LOCALAPPDATA%\M32\logs`
- crashes: `%LOCALAPPDATA%\M32\crashes`

The T011 startup path initializer creates directory entries only:

- root
- cache
- logs
- crashes

`config.json` is a file path contract. T011 does **not** create an empty configuration file.
Configuration schema/default serialization belongs to a later configuration Task.

## 3. Startup behavior

At startup M32:

1. reads `LOCALAPPDATA`;
2. builds the stable M32 path layout;
3. creates required directories recursively;
4. exits with code `2` if discovery or directory creation fails.

M32 does not silently fall back to the current directory because doing so could scatter user data
beside an executable or inside a repository checkout.

## 4. Logging privacy

The existing T009 INFO startup event does not print absolute runtime paths.

T011 emits only a DEBUG-level `runtime_paths_ready` state event without path fields.

Future diagnostic UI may display paths deliberately, but normal INFO logs must not expose the
user profile path by default.

## 5. Log and crash persistence

T011 reserves stable `logs` and `crashes` directories.

It does not yet add log rotation or write panic reports to files. Those features must consume these
stable directories when their dedicated persistence behavior is implemented.

This keeps T011 focused on path ownership and prevents hidden file-format/rotation contracts from
being introduced accidentally.

## 6. Portability

The v1.0 supported runtime target is Windows x64 MSVC.

Future macOS/Linux path conventions require their own platform-support Task and must not change the
Windows contract above without RFC/spec review.
