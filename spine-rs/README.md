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

This project is licensed under the [BSD 3-Clause License](LICENSE) (or <https://opensource.org/license/bsd-3-clause>).

In short, you’re free to use, modify, and distribute this software however you want, as long as you:

* Keep the original copyright notice and license text.
* Don’t use my name or the contributors’ names to promote forked products without permission.

Cheers, RazkarStudio.

Copyright © 2026 RazkarStudio and Spine Contributors. All rights reserved.
