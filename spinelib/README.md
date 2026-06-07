# spinelib

The Ergonomic Rust API for parsing Spine documents.

## Usage

```rust
use spinelib::Document;

let doc = Document::parse("server\n| host = localhost\n| port = 8080\n")?;
let host = doc.root()
    .and_then(|r| r.get("server"))
    .and_then(|s| s.get("host"))
    .and_then(|h| h.as_str());

println!("{host:?}"); // Some("localhost")
```

<details>
    <summary>Error Output</summary>

Spine uses a boxed-based error format, which share the same similarities to pipes if you squint hard enough.

```
┌─ error: duplicate-key
│  <input>
├─ 1:1 host = localhost
│      ^^^^ first defined here
├─ 2:1 host = example.com
│      ^^^^ redefined here
└─ 'host' was already defined
```

</details>

## License

This project is licensed under the [BSD 3-Clause License](LICENSE) (or <https://opensource.org/license/bsd-3-clause>).

In short, you’re free to use, modify, and distribute this software however you want, as long as you:

* Keep the original copyright notice and license text.
* Don’t use my name or the contributors’ names to promote forked products without permission.

Cheers, RazkarStudio.

Copyright © 2026 RazkarStudio and Spine Contributors. All rights reserved.
