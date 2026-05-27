# Spine

Spine is a line-oriented data format designed for readable, structured, and streaming-friendly configuration and data files.

It is not a programming language. It does not execute logic. It is purely a structured data representation format.

Spine is designed to feel natural to write by hand, easy to generate, and simple to parse consistently across languages.

---

## Why Spine exists

Most existing formats force tradeoffs:

- JSON is strict and noisy
- YAML is flexible but ambiguous
- TOML is readable but limited in structure

Spine tries to sit in a different space:

- explicit structure without indentation rules
- no inline nesting syntax
- predictable parsing behavior
- friendly for generated files and diffs
- easy to stream and append data incrementally

---

## Core idea

Spine represents structure using pipes (`|`) to indicate hierarchy.

Example:

```spine
server
| host = localhost
| port = 8080
````

This is equivalent to:

```spine
server.host = localhost
server.port = 8080
```

Both forms represent the same data.

---

## Values

Spine supports:

* strings
* numbers
* booleans (`true`, `false`)
* null
* objects
* arrays
* tagged values

---

## Arrays

Arrays use `-` for entries:

```spine
features
| - auth
| - sync
| - metrics
```

Objects inside arrays:

```spine
packages
| -
| | name = react
| | version = 19.0.0
```

---

## Appending

Arrays can be appended using `~`:

```spine
~packages
| name = react
```

This is useful for generated or incremental output.

---

## Strings

Strings can be written bare or quoted:

```spine
name = localhost
message = "hello world"
```

Quoted strings support escape sequences.

---

## Multiline strings

Multiline strings use triple quotes:

```spine
query = """
| SELECT *
| FROM users
| WHERE active = true
| """
```

When a multiline string begins, Spine remembers the current structural depth and strips matching leading pipes from each line. Everything else is preserved literally.

---

## Tagged values

Tagged values allow extensible typed data without changing the core format.

```spine
created = date"2026-05-26"
payload = base64"SGVsbG8="
```

Tags are not interpreted by the parser. They are preserved for later decoding by applications.

Tags may be namespaced:

```spine
std.date"2026-05-26"
```

---

## Comments

Line comments:

```spine
# this is a comment
```

Inline comments:

```spine
port = 8080 # default port
```

Block comments:

```spine
/*
multi-line comment
*/
```

Block comments may be nested.

---

## Merging rules

Objects merge structurally:

```spine
server.host = localhost

server
| port = 8080
```

This produces a single `server` object.

Duplicate scalar fields are not allowed.

---

## Errors

Spine is strict about structural consistency.

Invalid examples:

```spine
server = localhost
server.port = 8080
```

A value cannot be both a scalar and an object.

---

## Design goals

Spine is built around a few core principles:

* structure should be explicit and visible
* parsing should be deterministic
* output should remain stable across runs
* files should be easy to diff and generate
* syntax should stay simple enough for multiple language implementations

Spine intentionally avoids:

* inline object syntax
* inline array syntax
* indentation-based parsing rules
* runtime semantics or execution behavior

---

## Status

This is an early-stage format and Rust implementation. The syntax and rules may evolve as the parser is implemented and real-world edge cases are discovered.

---

# License

MIT OR Apache-2.0

Cheers, RazkarStudio

Copyright © 2026 RazkarStudio. All rights reserved.
