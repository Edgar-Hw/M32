# M32 RustJava Local Compatibility Patches

Base repository:

```text
https://github.com/dlunch/RustJava.git
```

Base revision:

```text
ba5797b8eb4cf376fdd63129903d319d1d7acf98
```

## M32-RJ-001 — RustJar caller must see application JAR classes

Task:

```text
0.0.3-T003
```

Patched file:

```text
jvm/src/jvm.rs
```

Pinned WIE `net/wie/Launcher` is defined by `RustJarClassLoader` and calls `jvm.new_class()` for the
guest MIDlet. Pinned RustJava resolves that request through the calling class's defining loader.

`RustJarClassLoader::findClass()` deliberately ignores all classpath entries that do not end in
`.rustjar`, so an ordinary guest `.jar` is never queried.

M32 keeps the exact RustJava base revision but applies a narrow compatibility rule:

```text
caller loader == org/rustjava/lang/RustJarClassLoader
-> resolve through the existing system URLClassLoader
```

Pinned RustJava constructs that system loader as:

```text
URLClassLoader(parent = RustJarClassLoader)
```

so native RustJar/WIE classes remain parent-resolved while application JARs become visible in the URL
layer.

Apply with:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\apply-rustjava-system-loader-compat.ps1
```

Verify with:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify-rustjava-system-loader-compat.ps1
```
