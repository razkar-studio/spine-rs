# Spine Language Specification v1.0

> **Status:** RELEASE CANDIDATE 3
> **Version:** 1.0
> **Encoding:** UTF-8
> **File extension:** `.spn`

---

## 1. Introduction

Spine is a hierarchical configuration language. Its design centers on three ideas:

- **Structure is explicit**: indentation is marked with `|` pipes, never with whitespace count alone.
- **Values are bare by default**: strings do not require quotes in value position; the parser infers type from content.
- **Errors accumulate**: a single parse pass reports all errors, not just the first one.

The remainder of this document defines Spine v1.0 in sufficient detail that any implementation claiming conformance must accept every well-formed document in this specification and reject every ill-formed document with errors matching those described.

---

## 2. Lexical Structure

### 2.1 Character Set

Spine source text is **UTF-8** encoded. The following characters carry syntactic meaning:

- **ASCII** code points U+0020–U+007E carry syntactic meaning (§2.1).
- **Unicode letters** (code points for which Rust's `char::is_alphabetic()` returns `true`) also carry syntactic meaning as identifier and bare-value characters.

All other non-ASCII Unicode codepoints appearing in comments, string literals, or bare values are preserved verbatim as content but have no syntactic role.

The following characters are significant:

| Character | Name | Role |
|-----------|------|------|
| `\n` | Newline | Statement terminator, resets pipe count |
| ` `, `\t` | Space, Tab | Ignored before pipes; significant inside values |
| `|` | Pipe | Indentation depth marker |
| `=` | Equals | Key-value assignment |
| `-` | Dash | Array element marker |
| `~` | Tilde | Append marker |
| `.` | Dot | Dotted key path separator |
| `#` | Hash | Line comment start |
| `"` | Double quote | String literal delimiter |
| `/` | Slash | Block comment start (when followed by `*`) |

All other ASCII printable characters, as well as Unicode letters (per Rust's `char::is_alphabetic()`), are valid in bare strings and identifiers unless otherwise specified.

### 2.2 Whitespace and Newlines

- **Newline** (`\n`) terminates a statement. Carriage return (`\r`) is treated as part of a bare value or string content.
- **Spaces and tabs** before the first pipe on a line are ignored. After the first non-whitespace character, they are significant.
- A blank line (zero pipes, empty content) is a no-op.
- A **final newline at EOF** is optional. If absent, the stream is treated as though it ends with a newline.

### 2.3 Comments

#### Line Comments

A line comment begins with `#` and extends to the end of the line (including EOF). Line comments are ignored by the parser.

```spine
key = value  # this is a comment
```

#### Block Comments

A block comment begins with `/*` and ends with `*/`. Block comments may span multiple lines. Comments may be **nested** — each nested `/*` increments a depth counter, and the comment is closed only when all levels are closed.

```spine
/* single-line */

/*
  multi-
  line
*/
```

An unclosed block comment at EOF is a lexical error: `unterminated block comment`.

### 2.4 Tokens

The lexer produces the following token types:

| Token | Produced by | Notes |
|-------|-------------|-------|
| `Pipe` | `|` at line start (after optional leading whitespace) | One per depth level |
| `Equals` | `=` | Sets the *bare-value state* for the next token |
| `Tilde` | `~` | Append operator |
| `Dash` | `-` | Array element marker; sets the *bare-value state* for the next token |
| `Dot` | `.` | Path separator |
| `Newline` | `\n` | Resets bare-value state and pipe count |
| `Ident(s)` | An identifier | See §2.5 |
| `Str(s)` | A quoted or multi-line string | See §3.4 |
| `Number(n)` | A numeric literal | See §3.3 |
| `Bool(b)` | `true` or `false` | Keywords |
| `Null` | `null` | Keyword |
| `Tagged(tag, content)` | `ident"..."` or `ident.ident"..."` | See §3.5 |
| `LineComment(s)` | `# ...` | Discarded |
| `BlockComment(s)` | `/* ... */` | Discarded |
| `Unknown(c)` | Any unexpected character | Indicates lex error |

### 2.5 Identifiers

An identifier starts with an underscore (`_`) or any character for which Rust's `char::is_alphabetic()` returns `true` (which includes ASCII letters `A–Z`, `a–z` and Unicode letters such as `é`, `ñ`, `β`, `あ`, etc.). This is followed by zero or more identifier-continuation characters: underscores, hyphens, any alphabetic character (per `is_alphabetic()`), or any ASCII digit (`0–9`).

```
identifier ::= id-start { id-continue }
id-start    ::= '_' | ? char::is_alphabetic() ?
id-continue ::= id-start | '-' | ? char::is_ascii_digit() ?
```

The hyphen is permitted inside identifiers to support common naming patterns such as `scram-sha-256` or `eu-central-1` when they appear as **keys** (not bare values). An identifier that is also a keyword (`true`, `false`, `null`) is recognized as that keyword.

### 2.6 Bare-Value State

The lexer maintains an internal flag called the *bare-value state*. This flag is:

- **Set to `true`** immediately after consuming `=` or `-`.
- **Set to `false`** immediately after consuming the value token, or upon encountering a newline.

When the bare-value state is active, the next token consumes **all remaining characters on the line** (until `\n`, `#`, or EOF) and type-inference rules are applied:

1. The complete text is captured.
2. Leading and trailing whitespace is trimmed.
3. If the trimmed text can be parsed as an `f64` number (matching the Number grammar in §3.3), the token is emitted as `Number(n)`.
4. Otherwise, the token is emitted as `Str(s)`.

This mechanism is what produces bare (unquoted) string values such as `localhost`, `https://telemetry.local`, and `scram-sha-256`, while still correctly typing pure numeric literals like `8080` and `3.14`.

> **Note:** After bare-value state consumes a token, the flag is cleared. Any subsequent `=` or `-` in the same line must be the start of a new statement on a new line. The flag is always reset by newline.

#### Bare vs Quoted Strings: Summary

These rules ensure that in a key–value pair the value never needs quoting, while quoted strings remain available when precise control (e.g., leading/trailing whitespace, multi-line content) is needed.

| Input | Token emitted | Reason |
|-------|---------------|--------|
| `= localhost` | `Str("localhost")` | Bare-value active → plain text |
| `= 8080` | `Number(8080)` | Bare-value active → parses as f64 |
| `= 16GB` | `Str("16GB")` | Bare-value active → does not parse as f64 |
| `= "localhost"` | `Str("localhost")` | Explicit quoted string (See §3.4) |
| `key` (as identifier) | `Ident("key")` | Bare-value NOT active |
| `eu-central-1` (as identifier) | `Ident("eu-central-1")` | Bare-value NOT active |

### 2.7 Escape Sequences in Quoted Strings

Inside `"..."` and `"""..."""` strings, the following escape sequences are recognized:

| Sequence   | Meaning                                                 |
| ---------- | ------------------------------------------------------- |
| `\n`       | Newline (U+000A)                                        |
| `\t`       | Horizontal tab (U+0009)                                 |
| `\r`       | Carriage return (U+000D)                                |
| `\0`       | Null character (U+0000)                                 |
| `\\`       | Backslash (U+005C)                                      |
| `\"`       | Double quote (U+0022)                                   |
| `\xNN`     | Character with hexadecimal value `NN` (00–FF)           |
| `\uXXXX`   | Unicode scalar value with hexadecimal code point `XXXX` |
| `\u{X...}` | Unicode scalar value with hexadecimal code point `X...` |

For `\xNN`, `NN` must consist of exactly two hexadecimal digits (`0-9`, `A-F`, `a-f`).

For `\uXXXX`, `XXXX` must consist of exactly four hexadecimal digits (`0-9`, `A-F`, `a-f`).

For `\u{X...}`, the braces must contain one or more hexadecimal digits (`0-9`, `A-F`, `a-f`) representing a Unicode scalar value.

Unicode escape sequences must resolve to a valid Unicode scalar value. Values outside the Unicode range or within the surrogate range (`U+D800`–`U+DFFF`) are a lexical error.

Any other character following a backslash is a lexical error.

---

## 3. Values

### 3.1 Null

```
null-value ::= 'null'
```

The keyword `null` represents the absence of a value. In array blocks, `-` followed by nothing (immediate newline) also produces `Null`.

### 3.2 Boolean

```
bool-value ::= 'true' | 'false'
```

### 3.3 Number

```
number    ::= ['-'] integer [ '.' fraction ]
integer   ::= digit { digit }
fraction  ::= digit { digit }
digit     ::= '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
```

Numbers are **IEEE 754 double-precision** floating-point values. A leading minus sign (`-`) is part of the number literal.

| Valid numbers | Invalid |
|---------------|---------|
| `0` | `+1` (leading plus) |
| `42` | `0xFF` (no hex) |
| `-42` | `1e10` (no scientific notation) |
| `3.14` | `1_000` (no underscores) |
| `-0.5` | `.5` (no leading dot) |

### 3.4 String

#### Quoted Strings

```
quoted-string ::= '"' { char | escape-sequence } '"'
```

A quoted string begins and ends with `"`. It may contain any character except an unescaped `"` or unescaped newline. An unescaped newline inside a `"..."` string is a lexical error: `unterminated string`.

An empty string `""` is valid and produces `Value::String("")`.

#### Multi-line Strings

```
multiline-string ::= '"""' newline
                     { line }
                     final-pipes '"""'
```

A multi-line string begins with `"""`, followed by a newline, then zero or more content lines, and ends with `"""` at the same pipe depth as the opening statement.

Each content line may include `|` pipes matching the current indentation depth (see §5.1). Leading pipe characters and the whitespace before them are stripped. Pipes not matching the opening depth are preserved verbatim.

```spine
query = """
| SELECT user_id, COUNT(*)
| FROM events
| GROUP BY user_id
| """
```

The content of this string is:

```
SELECT user_id, COUNT(*)
FROM events
GROUP BY user_id
```

The closing `"""` must appear at the same pipe depth as the opening `"""`. If it is missing before EOF, a lexical error is emitted: `unterminated multiline string`.

### 3.5 Tagged

```
tagged-value ::= qualified-ident string-content
qualified-ident ::= identifier { '.' identifier }
string-content ::= '"' { char | escape-sequence } '"'
```

A tagged literal is a qualified identifier immediately followed by a quoted string with no whitespace between them. The quoted string follows the same rules as a regular quoted string (§3.4).

```spine
created = date"2026-05-26"
hash = base64"c3BpbmUtZnVsbC1leGFtcGxl"
expires = std.date"2027-01-01"
```

A tagged literal produces `Value::Tagged(tag, content)` where `tag` is the qualified identifier text and `content` is the string content.

### 3.6 Array

```
array-value ::= array-block (see §5.5)
```

Arrays are defined via the dash syntax (§5.5). An array is an ordered sequence of values. There is no inline array syntax (e.g., `[1, 2, 3]` is **not** valid Spine).

### 3.7 Object

```
object-value ::= implicit-object (see §5.3) | key-value (see §5.2)
```

An object is an ordered collection of key–value pairs. **Insertion order is preserved** and guaranteed by conforming implementations. Keys are unique within an object (see §7.1 for duplicate-handling rules).

Objects can be defined implicitly (a bare key followed by indented children) or via dotted paths.

---

## 4. Top-Level Document

A Spine document is always an **Object** at the top level. It is parsed as follows:

```
document ::= { statement }
statement ::= key-value
            | implicit-object
            | dotted-path
            | append
```

Leading comments and blank lines are ignored. The parser collects all statements into a single root object.

---

## 5. Structure

### 5.1 Indentation

Indentation is marked by the `|` (pipe) character. Leading whitespace before the first pipe on a line is **ignored**, only the count of pipes matters. Each pipe represents one depth level relative to the parent.

```
depth = 0                                    # root level
| depth = 1                                  # child of root
| | depth = 2                                # grandchild
| | | depth = 3                              # etc.
```

The empty-pipe-depth is 0. A line with zero pipes at the start is at the root level.

**Indentation rules:**

1. A statement's children appear on subsequent lines with **exactly one more pipe** than the parent.
2. All children of the same parent must share the same pipe depth.
3. A return to a lower pipe depth closes the parent's scope.
4. White space before the first `|` is not significant, ` |` and `|` are equivalent. This means pipe alignment is purely cosmetic.

### 5.2 Key-Value Assignment

```
key-value ::= identifier '=' value
value     ::= null-value | bool-value | number | quoted-string
            | multiline-string | tagged-value
```

A key-value assignment assigns a scalar value to a key.

```
host = localhost
port = 8080
enabled = true
ttl = 300s
```

The value after `=` is evaluated using the bare-value rules (§2.6): a bare string is inferred from the remainder of the line, and pure number literals are typed as Number.

### 5.3 Implicit Objects

```
implicit-object ::= identifier newline
                    { pipe child-statement }
```

When a key is followed by a newline (not `=`), the parser checks the next line for indented children. If children exist, the key becomes an **Object** whose fields are the child statements.

```spine
server                    # implicit object "server"
| host = localhost        # child key-value
| port = 8080             # child key-value
```

This is equivalent to `server = { host = "localhost", port = 8080 }` in JSON-like notation.

### 5.4 Dotted Paths

```
dotted-path ::= identifier '.' identifier [ '.' identifier ... ]
```

A dot in a key name acts as a **path separator**, creating nested objects.

```spine
system.runtime.env = production
```

is equivalent to:

```spine
system
| runtime
| | env = production
```

Dots may appear in identifier-based syntax after `~` (§5.6), creating nested object structures on the append path. Dots as value content (e.g., in a bare string) are literal characters with no path semantics.

> **Note:** There is no escape mechanism for literal dots in key names. A dot in a key is always interpreted as a path separator.

### 5.5 Array Blocks

```
array-block ::= identifier newline
                { pipe dash-element }
dash-element ::= '-' value? newline
               | '-' implicit-object
```

An array block is defined by a key, a newline, and child lines beginning with `-` (dash) at the next depth level. Each dash introduces one array element.

#### Plain Elements

Each `-` followed by content creates a scalar array element. The same bare-value rules apply after `-` as after `=`:

```
regions
| - eu-central-1
| - eu-west-1
| - us-east-1
```

produces `regions = ["eu-central-1", "eu-west-1", "us-east-1"]`.

#### Object Elements

When a `-` is followed by a newline and indented children at one deeper pipe depth, the element becomes an Object.

```
features
| -
| | name = new-ui
| | enabled = true
| -
| | name = dark-mode
| | enabled = true
```

produces `features = [{ name = "new-ui", enabled = true }, { name = "dark-mode", enabled = true }]`.

#### Empty Elements

A `-` followed immediately by a newline (or EOF) produces `Null`.

### 5.6 Append

```
append ::= '~' dotted-path newline
           pipe child-statement
```

The `~` operator appends to an array. It is equivalent to a push operation.

```spine
~packages
| name = react
~packages
| name = vue
```

produces `packages = [{ name = "react" }, { name = "vue" }]`.

**Append semantics:**

1. If the key does not exist, create a new array containing the single child value.
2. If the key exists and is an array, push the child value onto the array.
3. If the key exists and is **not** an array, emit a `type-conflict` error (`"{key}" is not an array`).

Dotted paths after `~` navigate into existing nested objects:

```spine
server
| host = localhost
~server.users
| name = alice
~server.users
| name = bob
```

produces `server = { host = "localhost", users = [{ name = "alice" }, { name = "bob" }] }`.

If a path segment does not exist, an empty Object is auto-created for it. If a path segment exists but is not an Object, a `type-conflict` error is emitted.

---

## 6. Type System

### 6.1 Value Types

The Spine type system consists of seven types:

| Type | Runtime representation | Examples |
|------|------------------------|----------|
| `Null` | Unit / none | `null`, `-` (empty) |
| `Bool` | Boolean | `true`, `false` |
| `Number` | IEEE 754 f64 | `42`, `-3.14`, `0` |
| `String` | UTF-8 text | `"hello"`, `localhost`, `300s` |
| `Tagged` | Type-tagged string | `date"2026-01-01"`, `base64"..."` |
| `Array` | Ordered list | `[1, 2, 3]` (via `-`) |
| `Object` | Ordered map | `{ a = 1, b = 2 }` (via implicit object or `=`) |

### 6.2 Type Ordering

- **Objects** maintain **insertion order** of their fields. Conforming implementations must preserve this order.
- **Arrays** maintain the order of their elements as they appear in the source.
- **Object field ordering** is semantically significant (matching the principle of least surprise: what you write is what you get).

### 6.3 Tagged Literals as Structured Strings

Tagged literals are **not** a distinct semantic type, they are a pair of (tag, content) where both are strings. The tag serves as a hint to the consumer (e.g., `date`, `base64`, `std.date`) but carries no meaning at the parsing layer. Tagged literals are opaque to the parser.

---

## 7. Error Handling

Spine uses **error accumulation**: all errors are collected during a single parse pass. Parsing continues after each error, and all discovered errors are returned at the end.

### 7.1 Duplicate Key

If the same key is defined twice at the same object level:

- If **both values are Objects**, they are **merged recursively** (child keys are combined).
- Otherwise, a `duplicate-key` error is emitted for the second definition.

```
host = localhost
host = example.com          # error: duplicate-key
```

### 7.2 Type Conflict

If a key is defined as a scalar value and later used as an object, or vice versa:

```
server = localhost           # scalar
server
| port = 8080               # error: type-conflict (scalar vs object)
```

A `type-conflict` error is also emitted when `~` targets a non-array value, or when a path segment in an append path is not an object.

### 7.3 Lexical Errors

| Error | Condition |
|-------|-----------|
| `unterminated string` | `"..."` string ends at newline without closing `"` |
| `unterminated multiline string` | `"""..."""` without closing `"""` before EOF |
| `unterminated block comment` | `/* ...` without closing `*/` before EOF |
| `unexpected character '{c}'` | A character with no syntactic role appears |

### 7.4 Error Messages

Errors must include:

- The error type (`duplicate-key`, `type-conflict`, or lexical error name).
- A source location (line number, column).
- A message explaining the problem.
- For duplicate-key and type-conflict errors, the location of the **first definition**.

---

## 8. Formal Grammar

The following EBNF grammar defines the complete Spine v1.0 syntax.

```
(* -- Top Level -- *)
document       = { statement | comment | blank-line }
statement      = key-value | implicit-object | append

(* -- Key-Value -- *)
key-value      = identifier '=' value newline
value          = null-value
               | bool-value
               | number
               | quoted-string
               | multiline-string
               | tagged-value

(* -- Implicit Object -- *)
implicit-object = identifier newline
                  { pipe child-statement }
child-statement = statement (* at depth+1 *)

(* -- Dotted Append -- *)
append         = '~' path newline
                 { pipe child-statement }
path           = identifier { '.' identifier }

(* -- Array Block -- *)
array-block    = identifier newline
                 { pipe dash-element }
dash-element   = '-' newline               (* -> Null *)
               | '-' value newline         (* -> scalar *)
               | '-' newline               (* -> object *)
                 { pipe child-statement }

(* -- Literals -- *)
null-value     = 'null'
bool-value     = 'true' | 'false'
number         = [ '-' ] digit { digit } [ '.' digit { digit } ]
digit          = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'

(* -- Strings -- *)
quoted-string  = '"' { char | escape } '"'
multiline-string = '"""' newline
                   { line }
                   pipes '"""'
escape         = '\n' | '\t' | '\\' | '\"'

(* -- Tagged -- *)
tagged-value   = qualified-ident quoted-string
qualified-ident = identifier { '.' identifier }

(* -- Identifiers -- *)
identifier     = id-start { id-continue }
id-start       = '_' | ? char::is_alphabetic() ?
id-continue    = id-start | '-' | ? char::is_ascii_digit() ?

(* -- Indentation -- *)
pipe           = '|'
pipes          = { pipe }

(* -- Comments (discarded) -- *)
comment        = line-comment | block-comment
line-comment   = '#' { char } newline
block-comment  = '/*' { char | block-comment } '*/'

(* -- Implicit -- *)
newline        = '\n'
blank-line     = newline  (* but any number of pipes may appear *)
char           = ? any UTF-8 codepoint except newline and unescaped '"' ?
```

> **Note on bare values:** The grammar above shows `value` after `=` and `-`. The actual token consumed from the source may be a `Str`, `Number`, `Bool`, `Null`, or `Tagged` token. The bare-value inference described in §2.6 handles the conversion from raw source characters to typed tokens.

---

## Appendix A. Complete Example

The following file demonstrates all Spine features:

```spine
# Spine full feature showcase

# -- Key-value scalars --
app
| name = Spine Showcase System
| version = 0.1.0
| mode = production

# -- Nested objects (3+ levels deep) --
system
| runtime
| | env = production
| | region = eu-central-1
| | timezone = UTC
|
| limits
| | cpu = 8
| | memory = 16GB               # bare string (doesn't parse as number)
| | io = high
|
| logging
| | level = info
| | format = json
| | sinks
| | | - stdout                  # bare string in array
| | | - file                    # bare string in array
| | | | path = /var/log/spine.log
| | | | rotation = daily
|
| telemetry
| | enabled = true
| | exporter = otel
| | endpoint = https://telemetry.local

# -- Array of objects --
database
| replicas
| | -
| | | host = db.replica-1.local
| | | lag = low
| |
| | -
| | | host = db.replica-2.local
| | | lag = medium

features
| flags
| | -
| | | name = new-ui
| | | enabled = true
| | | rollout = 0.5
| | -
| | | name = dark-mode
| | | enabled = true
| | | rollout = 1.0

# -- Tagged literals --
meta
| created = std.date"2026-05-26"
| hash = base64"c3BpbmUtZnVsbC1leGFtcGxl"

# -- Append (~) --
| ~events
| | type = created
| | timestamp = 2026-05-26T00:00:00Z
| ~events
| | type = deployed
| | timestamp = 2026-05-26T18:00:00Z

# -- Dotted append --
server
| host = localhost
~server.users
| name = alice
~server.users
| name = bob

# -- Multiline string --
analytics
| query = """
| SELECT user_id, COUNT(*)
| FROM events
| WHERE active = true
| GROUP BY user_id
| """
```

---

## Appendix B. Error Examples

```spine
# type-conflict: scalar then object
server = localhost               # defines server as String
server                           # attempts to redefine as Object
| port = 8080                    # error: type-conflict

# duplicate-key
host = localhost
host = example.com               # error: duplicate-key

# type-conflict via append: not an array
mode = production                # defines mode as String
~mode                            # attempts to append
| env = staging                  # error: type-conflict

# duplicate-key deep in an object
database
| host = db.local
| host = db.remote               # error: duplicate-key
```

---

## Appendix C. Known Gaps & Implementation Notes

This section is informational, not part of the spec.

### C.1 Negative Numbers

The spec defines negative numbers (e.g., `-42`) as valid numeric literals (§3.3). The current reference lexer implementation treats a leading `-` after `=` or `-` as part of a bare string rather than as a negative number prefix. A conforming implementation should handle this correctly by attempting to parse the full bare-value text as a number (which includes the leading minus) before falling back to string.

### C.2 Negative numbers in key context

`-42` at the start of a line (not after `=` or `-`) is currently treated as an array dash. A negative number at root level is not meaningful in the current grammar. This is by design, there is no expression syntax that would require it.
