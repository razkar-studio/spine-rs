# spinelib

The Ergonomic Rust API for parsing Spine documents. Built on top of the `spine-c` C ABI.

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

MIT OR Apache-2.0

Cheers, RazkarStudio.

Copyright © 2026 RazkarStudio. All rights reserved.
