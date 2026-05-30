# Ports

Supported, official, or officially acknowledged ports for the Spine data format.

| Language |      Library Name     |  Version  |
|----------|-----------------------|-----------|
| Rust     | [`spinelib`](https://codeberg.org/razkar/spine-rs/src/branch/main/spinelib) | Unstable (no version) |
| C++      | [`spinelib`](https://codeberg.org/razkar/spinelib-cpp) | Unstable (v0.1.0) |

_(that's it currently...)_

---

# Contributing

## Overview

`spine-c` exposes the official C ABI for creating bindings to other languages.

Bindings should target:
- `include/spine.h`
- the generated shared libraries

## Supported library formats

- Linux/BSD: `.so`
- Windows: `.dll`
- macOS: `.dylib`

## CI artifacts

Prebuilt libraries are available from GitHub Actions artifacts/releases.
