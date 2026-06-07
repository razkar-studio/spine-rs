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
| `spine-abi` | C ABI for use from any language |

_(See their individual READMEs for more information!)_

## Status

Spine is in early development and APIs may change.

## Contributing

If you have free time and when Spine develops more, you are more than welcome to make a Spine port for any language that doesn't already have an [_official / officially acknowledged port_](PORTS.md) using this ABI!

## License

The Spec
--------

The Spine Format Specification is licensed under the Creative Commons Attribution 4.0 International License (CC BY 4.0), to get a copy visit <https://creativecommons.org/licenses/by/4.0/>.

In short, you’re free to share, modify, and use the work however you want, even commercially, as long as you give proper credit and mention if you made changes.

The Rust Crates
---------------

The three Rust crates in this workspace are licensed under the [BSD 3-Clause License](LICENSE) (or <https://opensource.org/license/bsd-3-clause>).

In short, you’re free to use, modify, and distribute this software however you want, as long as you:

* Keep the original copyright notice and license text.
* Don’t use my name or the contributors’ names to promote forked products without permission.

Cheers, RazkarStudio

Copyright © 2026 RazkarStudio. All rights reserved.
