# Spine

Spine (`.spn`) is a structured data format designed to be readable, writable, and easy to process. It focuses on explicit structure, deterministic parsing, and good tooling support without becoming a programming language.

The format is line-oriented. Every meaningful statement exists on its own line, and hierarchy is expressed using leading pipe characters rather than indentation or braces.

Spine is intended for things like:

* configuration files
* lockfiles
* metadata
* generated structured data
* tooling-oriented formats

It is not intended to contain executable logic or runtime behavior.

---

# Structure

Spine represents hierarchy using leading pipes.

For example:

```spine
server
| host = localhost
| port = 8080
```

This defines an object named `server` with two fields.

The number of leading pipes indicates structural depth. Spaces are ignored for structure and may be used freely for readability.

The following form is equivalent:

```spine
server.host = localhost
server.port = 8080
```

Nested declarations and dotted paths are interchangeable and produce the same result.

---

# Values

Spine supports:

* objects
* arrays
* strings
* numbers
* booleans
* null
* tagged literals

Bare values are allowed.

```spine
host = localhost
port = 8080
enabled = true
```

The literals `true`, `false`, and `null` have special meaning. Numeric literals are parsed as numbers. Any other bare value is treated as a string.

Strings may also be quoted explicitly:

```spine
message = "hello world"
```

Quoted strings support escape sequences.

---

# Arrays

Arrays are represented using `-`.

```spine
features
| - auth
| - sync
| - metrics
```

Array entries may also contain objects.

```spine
packages
| -
| | name = react
| | version = 19.0.0
```

---

# Appending to Arrays

Spine includes explicit append syntax for arrays using `~`.

```spine
~packages
| name = react
```

This appends a new object entry into the `packages` array.

Append paths may be absolute:

```spine
~packages
```

or relative to the current scope:

```spine
| ~packages
```

This makes Spine especially suitable for generated files and incremental output.

---

# Multiline Strings

Multiline strings use triple quotes.

```spine
query = """
| SELECT *
| FROM users
| """
```

When a multiline string begins, Spine remembers the current structural depth. That exact number of leading pipes is stripped from subsequent lines, and all remaining content is preserved literally.

This allows multiline content to remain visually aligned with surrounding structure without affecting the resulting value.

Comments are not parsed inside multiline strings.

---

# Tagged Literals

Tagged literals allow applications to attach semantic meaning to values without changing the core format.

```spine
created = date"2026-05-26"
payload = base64"SGVsbG8="
```

The parser itself does not interpret these values. It only preserves the tag and its contents structurally. Applications may choose how to handle them.

Tags may also be namespaced.

```spine
std.date"2026-05-26"
```

---

# Comments

Line comments begin with `#`.

```spine
# generated automatically
```

Inline comments are also supported.

```spine
port = 8080 # default port
```

Block comments use `/* */`.

```spine
/*
multi-line
comment
*/
```

Block comments may nest.

---

# Merging and Conflicts

Objects merge structurally across declarations.

```spine
server.host = localhost

server
| port = 8080
```

This produces a single `server` object containing both fields.

Duplicate scalar fields are invalid.

```spine
host = localhost
host = example.com
```

This is an error.

Structural conflicts are also invalid.

```spine
server = localhost
server.port = 8080
```

A value cannot simultaneously be both a scalar and an object.

---

# Ordering

Field order is preserved.

Arrays are ordered.

This allows Spine documents to remain stable and readable when generated automatically.

---

# Design Philosophy

Spine is designed around a few core ideas.

Structure should be explicit and visually obvious. Parsing behavior should be deterministic and free from surprising coercion rules. Generated files should remain readable and produce stable diffs. Tooling should be easy to build, and the syntax should remain practical in terminals, editors, and source control.

The format intentionally avoids:

* indentation-sensitive parsing
* inline object syntax
* inline array syntax
* executable behavior
* implicit runtime semantics

Spine aims to stay small, predictable, and easy to implement across languages.
