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

## License

MIT OR Apache-2.0

Cheers, RazkarStudio.

Copyright © 2026 RazkarStudio. All rights reserved.
