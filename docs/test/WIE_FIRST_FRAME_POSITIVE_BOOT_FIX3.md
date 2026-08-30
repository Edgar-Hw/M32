# M32 0.0.3-T003 FIX3 — RustJar/System URLClassLoader root-cause fix

## Proven failure

FIX2 proved:

```text
real WIE FilesystemOverlay JAR round-trip = PASS
real J2ME boot = NoClassDefFoundError
filesystem calls = []
```

The empty filesystem trace means the guest JAR was never queried by the class loader.

## Exact source chain

Pinned WIE:

```text
net/wie/Launcher::start
-> jvm.new_class(main_class, "()V", ())
```

Pinned RustJava:

```text
Jvm::new_class
-> instantiate_class
-> resolve_class
-> current_class_loader
```

`current_class_loader()` uses the calling Java class's defining loader.

For `net/wie/Launcher`, that loader is:

```text
org/rustjava/lang/RustJarClassLoader
```

Pinned `RustJarClassLoader::findClass()` skips every classpath item that does not end in:

```text
.rustjar
```

Therefore the guest `.jar` is invisible in the exact call path used by WIE Launcher.

## Fix

When the current caller loader is exactly `RustJarClassLoader`, the M32 vendored RustJava
compatibility patch returns the already-created system loader.

Pinned RustJava's system loader is:

```text
URLClassLoader(parent = RustJarClassLoader)
```

This gives the intended two-layer behavior:

```text
WIE/MIDP rustjar classes -> RustJar parent
guest application JAR   -> URLClassLoader
```

## Verification strengthening

The positive boot test now requires both:

```text
M32_FIRST_FRAME_BOOT_OK sentinel
SessionState::Running
```

and an observed persistent-host metadata probe for:

```text
size:M32 Running Smoke:j2me-first-frame-running.jar
```

The persistent result remains `None`; WIE then falls through to its exact virtual 522-byte JAR.

This proves the URL class-loader path actually reached the guest archive.

## Scope

No WIE source edit.

No WIE revision change.

No RustJava base revision change.

No JAD/JAR/class byte change.

No dependency/Cargo.lock change.

The only production dependency-source modification is the documented local compatibility patch on
top of the already-vendored exact RustJava base revision.

## Status

```text
0.0.3-T003 = FIX3 / IN_PROGRESS
```
