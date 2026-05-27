# spine-rs

The canonical Spine parser. Lexer, parser, and core value types for the Spine data format.

This is the engine! Most users should use `spinelib` instead.

## Usage

```rust
use spine_rs::{Lexer, Parser};

let tokens = Lexer::new("host = localhost\n").tokenize();
let value = Parser::new(tokens, "host = localhost\n").parse();
```

## Errors

Errors are collected and returned as formatted, colored strings powered by [`farben`](https://github.com/razkar-studio/farben) ready to print.

## License

MIT OR Apache-2.0

Cheers, RazkarStudio.

Copyright © 2026 RazkarStudio. All rights reserved.
