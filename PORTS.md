# Ports

Supported, official, or officially acknowledged ports for the Spine data format.

| Language |      Library Name     |  Version  |
|----------|-----------------------|-----------|
| Rust     | [`spinelib`](https://codeberg.org/razkar/spine-rs/src/branch/main/spinelib) | Unstable (no version, unpublished) |
| C++      | [`spinelib`](https://codeberg.org/razkar/spinelib-cpp) | Unstable (v0.1.0, unpublished) |
| Python   | [`spinelib`](https://codeberg.org/razkar/pyspinelib) | Unstable (no version, unpublished) |
| Odin     | [`spinelib`](https://codeberg.org/razkar/spinelib-odin) (spinelib-odin/spinelib) | Unstable (v0.1.0, unpublished)

_(that's it currently...)_

> [!NOTE]
> i do not want to make a C port, just use the abi. i am out of languages i know

---

# Contributing

## Overview

`spine-abi` exposes the official ABI for creating bindings to other languages.

Bindings should target:
- `include/spine.h`
- the generated shared libraries or WASM module

## Supported library formats

### Native (shared libraries)
- Linux/BSD: `.so` (musl and glibc, x86_64 and aarch64)
- Windows: `.dll` (x86_64 and aarch64)
- macOS: `.dylib` (universal: x86_64 + aarch64)

### Portable (WASM)
- **Any platform with a WASM runtime**: `spine_abi.wasm`
  Works in browsers (via `wasm-bindgen`), wasmtime, wasmer, etc.

## CI artifacts

Prebuilt libraries are available from GitHub Actions artifacts/releases:

- **Native**: individual per-target downloads (`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-musl`, etc.)
- **Portable**: `wasm32-unknown-unknown`: runs in any WASM runtime
- **All-in-one**: `spine-abi-all` bundle containing every artifact plus `spine.h`

> WASM builds are a separate distribution channel from native shared libraries.
> They exist for environments where native linking is unavailable or impractical,
> providing a portable fallback without replacing the native ABI matrix.
