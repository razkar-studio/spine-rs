# Spine

Spine (`.spn`) is a structured data format designed to be readable, writable, and easy to process.

```spine
server
| host = localhost
| port = 8080
```

Spine uses leading pipes for hierarchy, which is explicit, visually obvious, and easy to generate programmatically.

## Crates

| Crate | Description |
|-------|-------------|
| `spine-rs` | The canonical Rust parser and lexer |
| `spinelib` | Ergonomic Rust API for reading Spine documents |
| `spine-c` | C ABI for use from any language |

_(See their individual READMEs for more information!)_

> [!NOTE]
> Fun fact, `spinelib` only depends on `spine-c` which proves that the ABI works. Still, you shouldn't use the ABI unless you're contributing to making a Spine port for another language!

## Status

Spine is in early development and APIs may change.

## Contributing

If you have free time and when Spine develops more, you are more than welcome to make a Spine port for any language that doesn't already have an [_official / officially acknowledged port_](PORTS.md) using this ABI!

## License

MIT OR Apache-2.0

Cheers, RazkarStudio

Copyright © 2026 RazkarStudio. All rights reserved.
