# Changelog

## 0.1.0+spec-1.0.0 — 2026-06-07

### Added

- Spec-compliant Spine parser (`spine-rs`) with lexer, AST, error accumulation, and 168 tests
- Serde integration layer (`spinelib`): serialize/deserialize Rust types to/from Spine documents, format/write support, 58 tests
- C ABI (`spine-abi`): 20 `extern "C"` functions for cross-language interop, with CI producing `.so`, `.dylib`, `.dll`, `.a`, `.wasm` artifacts
- Multi-platform CI: Linux x86_64/aarch64, Windows x86_64/aarch64, macOS universal, iOS, WASM, Android
- Full public API documentation with `# Errors`, `# Safety`, `# Panics`, `# Examples`
- SPEC.md matching RC3 of the Spine specification
