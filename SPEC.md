# Spine Specification
Version 1.0.0

License
-------

This specification is licensed under the Creative Commons Attribution 4.0 International License (CC BY 4.0).

© 2026 RazkarStudio and Spine Contributors.

To view a copy of this license, visit:
<https://creativecommons.org/licenses/by/4.0/>

Meta
----

+ Version: 1.0.0
+ Encoding: UTF-8
+ File Extension: `.spn`

---

## 1. Introduction

Spine is a hierarchical configuration language. Its design centers on three principles:

- **Structure is explicit**: indentation depth is signaled by `|` pipe characters, never by whitespace count alone.
- **Values are bare by default**: string values do not require quotation marks; the lexer infers the type from the literal content.
- **Errors accumulate**: a single parse pass collects all errors and reports them together, not just the first one encountered.

This document defines the Spine 1.0 format in sufficient detail that a conforming implementation must accept every well-formed document shown herein and reject every ill-formed document with errors of the kind described.

---

## 2. Lexical Structure

### 2.1 Character Set

Spine source text MUST be encoded as UTF-8. Two classes of code points carry syntactic meaning:

- ASCII code points U+0020–U+007E, as enumerated in the character table below.
- Unicode letters — code points for which `char::is_alphabetic()` returns `true` — which are valid in identifiers and bare values.

All other non-ASCII code points appearing inside comments, quoted strings, or bare values are preserved verbatim as content and carry no syntactic role.

The following characters are syntactically significant:

| Character | Name | Role |
|-----------|------|------|
| `\n` | Newline | Statement terminator; resets pipe depth |
| ` `, `\t` | Space, Tab | Ignored before pipes; significant inside values |
| `\|` | Pipe | Indentation depth marker |
| `=` | Equals | Key–value separator |
| `-` | Dash | Array element marker |
| `~` | Tilde | Append operator |
| `.` | Dot | Key-path separator |
| `#` | Hash | Line comment introducer |
| `"` | Double quote | String literal delimiter |
| `/` | Slash | Block comment introducer (when immediately followed by `*`) |

All other printable ASCII characters, as well as Unicode letters, are valid in bare strings and identifiers unless otherwise restricted by this specification.

### 2.2 Line Endings and Whitespace

A **newline** (`U+000A`, LINE FEED) terminates a statement. Carriage return (`U+000D`) is not treated as a line terminator; it is treated as content within a bare value or quoted string.

Spaces and tabs occurring before the first pipe character on a line are ignored. Once the first non-whitespace, non-pipe character is encountered, whitespace is significant.

A blank line — one containing no pipes and no content — is a no-op and is silently ignored.

A document that does not end with a newline is treated as though a final newline is present.

### 2.3 Comments

#### 2.3.1 Line Comments

A line comment is introduced by `#` and extends to the end of the line. Line comments are discarded by the lexer and have no effect on the parsed value.

```spine
key = value  # this is a comment
```

#### 2.3.2 Block Comments

A block comment is introduced by `/*` and terminated by `*/`. Block comments may span multiple lines. Nesting is supported: each `/*` increments a depth counter, and the comment closes only when all open levels have been closed by a corresponding `*/`.

```spine
/* single-line block comment */

/*
  multi-line
  block comment
*/

/* outer /* nested */ still in comment */
```

An unclosed block comment at end-of-file is a lexical error: `unterminated block comment`.

### 2.4 Token Types

The lexer produces the following token types:

| Token | Source | Notes |
|-------|--------|-------|
| `Pipe` | `\|` at line start (after optional leading whitespace) | One token per depth level |
| `Equals` | `=` | Activates bare-value state (§2.6) |
| `Tilde` | `~` | Append operator |
| `Dash` | `-` | Array element marker; activates bare-value state (§2.6) |
| `Dot` | `.` | Path separator |
| `Newline` | `\n` | Resets bare-value state and pipe depth |
| `Ident(s)` | An identifier | See §2.5 |
| `Str(s)` | A quoted or multi-line string literal | See §3.4 |
| `Number(n)` | A numeric literal | See §3.3 |
| `Bool(b)` | `true` or `false` | Reserved keywords |
| `Null` | `null` | Reserved keyword |
| `Tagged(tag, content)` | `ident"..."` or `ident.ident"..."` | See §3.5 |
| `LineComment(s)` | `# …` | Discarded |
| `BlockComment(s)` | `/* … */` | Discarded |
| `Unknown(c)` | Any unrecognized character | Indicates a lexical error |

### 2.5 Identifiers

An identifier begins with an underscore (`_`) or any character for which `char::is_alphabetic()` returns `true`. This includes ASCII letters `A`–`Z` and `a`–`z`, as well as Unicode letters such as `é`, `ñ`, `β`, and `あ`. Subsequent characters may be underscores, hyphens (`-`), or any character for which `char::is_alphanumeric()` returns `true`.

```
identifier ::= id-start { id-continue }
id-start    ::= '_' | ? char::is_alphabetic() ?
id-continue ::= id-start | '-' | ? char::is_alphanumeric() ?
```

The hyphen is permitted inside identifiers to accommodate common key patterns such as `scram-sha-256` and `eu-central-1`. An identifier that matches a reserved keyword (`true`, `false`, `null`) is emitted as the corresponding keyword token rather than as `Ident`.

### 2.6 Bare-Value State

The lexer maintains an internal one-bit flag called the *bare-value state*.

The flag is **set** immediately after the lexer consumes `=` or `-`. The flag is **cleared** immediately after the value token is emitted, or upon encountering a newline.

While the bare-value state is active, the lexer captures all remaining characters on the line (up to but not including `\n`, `#`, or EOF) as a single token, then applies the following type-inference rules in order:

1. Capture the full remaining text.
2. Strip leading and trailing whitespace.
3. If the stripped text is a valid number literal per the grammar in §3.3, emit `Number(n)`.
4. Otherwise, emit `Str(s)`.

This mechanism produces unquoted string values such as `localhost`, `https://telemetry.local`, and `scram-sha-256`, while correctly typing pure numeric literals such as `8080` and `3.14`.

> **Implementation note:** After the bare-value state produces a token, the flag is cleared. The newline always resets the flag unconditionally.

#### 2.6.1 Bare vs. Quoted Strings

The following table summarizes lexer behavior in and out of bare-value state:

| Source text | Bare-value active | Token emitted | Reason |
|-------------|:-----------------:|---------------|--------|
| `= localhost` | yes | `Str("localhost")` | Plain text; does not parse as number |
| `= 8080` | yes | `Number(8080)` | Parses as IEEE 754 f64 |
| `= 16GB` | yes | `Str("16GB")` | Does not parse as number |
| `= "localhost"` | yes | `Str("localhost")` | Explicit quoted string (§3.4) |
| `key` | no | `Ident("key")` | Identifier context |
| `eu-central-1` | no | `Ident("eu-central-1")` | Identifier context |

### 2.7 Escape Sequences

The following escape sequences are recognized inside `"…"` and `"""…"""` string literals:

| Sequence | Unicode code point |
|----------|--------------------|
| `\n` | U+000A LINE FEED |
| `\t` | U+0009 CHARACTER TABULATION |
| `\r` | U+000D CARRIAGE RETURN |
| `\0` | U+0000 NULL |
| `\\` | U+005C REVERSE SOLIDUS |
| `\"` | U+0022 QUOTATION MARK |
| `\xNN` | Code point with hexadecimal value `NN` (range 00–FF) |
| `\uXXXX` | Unicode scalar value with four-digit hex code point |
| `\u{X…}` | Unicode scalar value with one-or-more-digit hex code point |

For `\xNN`, `NN` MUST consist of exactly two hexadecimal digits (`0`–`9`, `A`–`F`, `a`–`f`).

For `\uXXXX`, `XXXX` MUST consist of exactly four hexadecimal digits.

For `\u{X…}`, the braces MUST contain at least one hexadecimal digit and MUST resolve to a valid Unicode scalar value.

Values within the surrogate range (`U+D800`–`U+DFFF`), or above `U+10FFFF`, are a lexical error.

Any character following a backslash that is not listed above is a lexical error.

---

## 3. Values

### 3.1 Null

```
null-value ::= 'null'
```

The keyword `null` represents the explicit absence of a value. In an array block, a `-` followed immediately by a newline also yields `Null`.

### 3.2 Boolean

```
bool-value ::= 'true' | 'false'
```

`true` and `false` are reserved keywords. They MUST be written in lowercase. No other casing is recognized as a boolean value.

### 3.3 Number

```
number    ::= [ '-' ] integer [ '.' fraction ]
integer   ::= digit { digit }
fraction  ::= digit { digit }
digit     ::= '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
```

Numbers are represented as IEEE 754 double-precision floating-point values. A leading minus sign is part of the literal. The following forms are explicitly not valid:

| Form | Example | Reason |
|------|---------|--------|
| Leading plus sign | `+1` | Not permitted |
| Hexadecimal notation | `0xFF` | Not supported |
| Scientific notation | `1e10` | Not supported |
| Underscore separators | `1_000` | Not supported |
| Leading decimal point | `.5` | Not permitted |

### 3.4 String

#### 3.4.1 Quoted Strings

```
quoted-string ::= '"' { char | escape-sequence } '"'
```

A quoted string is delimited by `"` on both ends. It may contain any character except an unescaped `"` or an unescaped newline. An unescaped newline inside a `"…"` string is a lexical error: `unterminated string`.

An empty quoted string `""` is valid and produces an empty string value.

#### 3.4.2 Multi-line Strings

```
multiline-string ::= '"""' newline
                     { line }
                     final-pipes '"""'
```

A multi-line string is introduced by `"""` followed immediately by a newline. Content continues on subsequent lines until a closing `"""` appears at the same pipe depth as the opening statement.

Each content line may be prefixed with pipe characters matching the current indentation depth (§5.1). Such leading pipes, along with any whitespace preceding them, are stripped from the content. Pipe characters in excess of the indentation depth are preserved verbatim as content.

```spine
query = """
| SELECT user_id, COUNT(*)
| FROM events
| GROUP BY user_id
| """
```

The value of the above string is:

```
SELECT user_id, COUNT(*)
FROM events
GROUP BY user_id
```

The closing `"""` MUST appear at the same pipe depth as the opening statement. A missing closing `"""` before end-of-file is a lexical error: `unterminated multiline string`.

### 3.5 Tagged Values

```
tagged-value    ::= qualified-ident string-content
qualified-ident ::= identifier { '.' identifier }
string-content  ::= '"' { char | escape-sequence } '"'
```

A tagged value is a qualified identifier immediately followed by a quoted string, with no whitespace between them. The quoted string follows the same rules as §3.4.1.

```spine
created = date"2026-05-26"
hash    = base64"c3BpbmUtZnVsbC1leGFtcGxl"
expires = std.date"2027-01-01"
```

A tagged value produces `Value::Tagged(tag, content)`, where `tag` is the full qualified identifier text and `content` is the parsed string. The tag is opaque to the parser; no interpretation is applied at the parsing layer.

### 3.6 Arrays

```
array-value ::= array-block   (see §5.5)
```

Arrays are defined using dash-block syntax (§5.5). An array is an ordered sequence of values. There is no inline array syntax; a construct such as `[1, 2, 3]` is not valid Spine.

### 3.7 Objects

```
object-value ::= implicit-object   (see §5.3)
               | key-value         (see §5.2)
```

An object is an ordered collection of key–value pairs. Insertion order MUST be preserved by conforming implementations. Keys within an object MUST be unique (see §7.1).

---

## 4. Document Structure

A Spine document is always an **Object** at the top level. The parser collects all top-level statements into a single root object. Leading comments and blank lines are discarded.

```
document  ::= { statement }
statement ::= key-value
            | implicit-object
            | dotted-path
            | append
```

A document with no statements is valid and produces an empty object.

---

## 5. Structural Elements

### 5.1 Indentation

Indentation is indicated exclusively by `|` (pipe) characters. Leading whitespace before the first pipe on a line is ignored; only the count of pipes determines the depth. Each pipe represents one level of nesting relative to the parent.

```spine
depth-0-key = value              # root (depth 0)
| depth-1-key = value            # child of root (depth 1)
| | depth-2-key = value          # grandchild (depth 2)
| | | depth-3-key = value        # great-grandchild (depth 3)
```

The root level has pipe depth 0. The following rules govern indentation:

1. Children of a statement appear on subsequent lines with exactly one more pipe than their parent.
2. All children of a given parent MUST share the same pipe depth.
3. A return to a shallower pipe depth closes the enclosing scope.
4. Whitespace preceding the first `|` on a line is not significant: `  |` and `|` are equivalent. Pipe alignment is cosmetic only.

### 5.2 Key–Value Pairs

```
key-value ::= identifier '=' value newline
value     ::= null-value | bool-value | number | quoted-string
            | multiline-string | tagged-value
```

A key–value pair binds a scalar value to a key within the enclosing object.

```spine
host    = localhost
port    = 8080
enabled = true
ttl     = 300s
```

The value following `=` is processed under bare-value state (§2.6): the remainder of the line is captured and typed as either `Number` or `String` by inference. Quoted and multi-line strings bypass bare-value inference.

### 5.3 Implicit Objects

```
implicit-object ::= identifier newline
                    { pipe child-statement }
```

When a key appears on a line without `=`, the parser examines the following lines for indented children. If one or more children exist at pipe depth `n+1`, the key is defined as an **Object** whose fields are those child statements.

```spine
server
| host = localhost
| port = 8080
```

This is semantically equivalent to `{ "server": { "host": "localhost", "port": 8080 } }` in JSON notation.

A bare key with no indented children is a no-op for the purposes of value definition.

### 5.4 Dotted Paths

```
dotted-path ::= identifier { '.' identifier }
```

A dot in a key position acts as a path separator, creating or descending into nested objects. The following two representations are equivalent:

```spine
system.runtime.env = production
```

```spine
system
| runtime
| | env = production
```

Dots may appear in the path following `~` (§5.6) to target a nested location. A dot appearing inside a bare string value is a literal character with no path semantics.

There is no mechanism for escaping a literal dot in a key name. A dot in a key is always interpreted as a path separator.

### 5.5 Array Blocks

```
array-block  ::= identifier newline
                 { pipe dash-element }
dash-element ::= '-' value newline
               | '-' newline { pipe child-statement }
               | '-' newline
```

An array block is introduced by a key followed by a newline, with subsequent lines at depth `n+1` each beginning with `-`. Each `-` introduces one element of the array.

#### 5.5.1 Scalar Elements

A `-` followed by content on the same line produces a scalar element. Bare-value state applies after `-` in the same way it applies after `=`:

```spine
regions
| - eu-central-1
| - eu-west-1
| - us-east-1
```

produces `regions = ["eu-central-1", "eu-west-1", "us-east-1"]`.

#### 5.5.2 Object Elements

A `-` followed by a newline, with further indented children at depth `n+2`, produces an Object element:

```spine
features
| -
| | name    = new-ui
| | enabled = true
| -
| | name    = dark-mode
| | enabled = true
```

produces `features = [{ "name": "new-ui", "enabled": true }, { "name": "dark-mode", "enabled": true }]`.

#### 5.5.3 Null Elements

A `-` followed immediately by a newline (or end-of-file), with no indented children, produces `Null`.

### 5.6 Append

```
append ::= '~' dotted-path newline
           { pipe child-statement }
```

The `~` operator appends one element to an array. It is semantically a push onto the named array.

```spine
~packages
| name = react
~packages
| name = vue
```

produces `packages = [{ "name": "react" }, { "name": "vue" }]`.

The semantics of `~` are as follows:

1. If the target key does not exist, a new array is created containing the single appended value.
2. If the target key exists and is an array, the value is pushed onto the end of the array.
3. If the target key exists and is not an array, a `type-conflict` error is emitted: `"<key>" is not an array`.

When a dotted path follows `~`, each segment is traversed into the existing object hierarchy:

```spine
server
| host = localhost
~server.users
| name = alice
~server.users
| name = bob
```

produces `{ "server": { "host": "localhost", "users": [{ "name": "alice" }, { "name": "bob" }] } }`.

If a path segment in the dotted path does not exist, an empty Object is implicitly created for it. If a path segment exists but is not an Object, a `type-conflict` error is emitted.

---

## 6. Type System

### 6.1 Value Types

Spine defines seven value types:

| Type | Description | Examples |
|------|-------------|----------|
| `Null` | Absence of a value | `null`, bare `-` |
| `Bool` | Boolean | `true`, `false` |
| `Number` | IEEE 754 double-precision float | `42`, `-3.14`, `0` |
| `String` | UTF-8 text | `"hello"`, `localhost`, `300s` |
| `Tagged` | Type-tagged string pair | `date"2026-01-01"`, `base64"…"` |
| `Array` | Ordered sequence of values | Defined via `-` syntax |
| `Object` | Ordered map of key–value pairs | Defined via implicit object or `=` |

### 6.2 Ordering Guarantees

Array elements are ordered by their position of appearance in the source document. Object fields are ordered by insertion order — the order in which they first appear in the source. Conforming implementations MUST preserve both orderings. This guarantee is semantically significant: the parsed value MUST reflect the order in which keys and elements are written.

### 6.3 Tagged Values

A tagged value is not a distinct semantic type at the parsing layer. It is a pair `(tag: String, content: String)` where `tag` is the qualified identifier text and `content` is the quoted string body. The parser attaches no meaning to the tag; interpretation is the responsibility of the consuming application.

---

## 7. Error Handling

Spine parsers MUST implement error accumulation: all errors encountered during a single parse pass are collected and reported together. The parser MUST continue after each error in order to discover subsequent errors. A conforming implementation MUST NOT stop at the first error.

### 7.1 Duplicate Keys

When the same key is defined more than once within the same object scope:

- If both definitions are **Objects**, the objects are **merged recursively**: child keys from the second definition are combined with those of the first.
- In all other cases, a `duplicate-key` error is emitted for the second definition.

```spine
host = localhost
host = example.com    # error: duplicate-key
```

### 7.2 Type Conflicts

A `type-conflict` error is emitted when a key is first established with one type and subsequently used in a way that is incompatible with that type.

```spine
server = localhost    # server is String
server               # attempts to treat server as Object
| port = 8080        # error: type-conflict
```

A `type-conflict` error is also emitted when `~` targets a key that is not an Array, or when a path segment in an append path is not an Object.

### 7.3 Lexical Errors

| Error | Condition |
|-------|-----------|
| `unterminated string` | A `"…"` literal reaches a newline without a closing `"` |
| `unterminated multiline string` | A `"""…"""` literal reaches end-of-file without a closing `"""` |
| `unterminated block comment` | A `/*` block reaches end-of-file without a closing `*/` |
| `unexpected character '<c>'` | A character appears with no syntactic role in the current context |

### 7.4 Error Reporting Requirements

Every error MUST include:

- The error kind (`duplicate-key`, `type-conflict`, or the applicable lexical error name).
- A source location, expressed as line number and column number.
- A human-readable message describing the problem.
- For `duplicate-key` and `type-conflict` errors: the source location of the **first definition** of the conflicting key.

---

## 8. Formal Grammar

The following EBNF grammar defines the complete Spine 1.0 syntax.

```ebnf
(* ── Top level ──────────────────────────────────────── *)
document        = { statement | comment | blank-line }
statement       = key-value | implicit-object | append

(* ── Key–value ──────────────────────────────────────── *)
key-value       = identifier '=' value newline
value           = null-value
                | bool-value
                | number
                | quoted-string
                | multiline-string
                | tagged-value

(* ── Implicit object ────────────────────────────────── *)
implicit-object = identifier newline
                  { pipe child-statement }
child-statement = statement   (* at depth + 1 *)

(* ── Append ─────────────────────────────────────────── *)
append          = '~' path newline
                  { pipe child-statement }
path            = identifier { '.' identifier }

(* ── Array block ────────────────────────────────────── *)
array-block     = identifier newline
                  { pipe dash-element }
dash-element    = '-' newline                  (* → Null *)
                | '-' value newline            (* → scalar *)
                | '-' newline                  (* → object *)
                  { pipe child-statement }

(* ── Literals ───────────────────────────────────────── *)
null-value      = 'null'
bool-value      = 'true' | 'false'
number          = [ '-' ] digit { digit } [ '.' digit { digit } ]
digit           = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'

(* ── Strings ────────────────────────────────────────── *)
quoted-string    = '"' { char | escape } '"'
multiline-string = '"""' newline
                   { line }
                   pipes '"""'
escape           = '\n' | '\t' | '\r' | '\0' | '\\' | '\"'
                 | '\x' hex hex
                 | '\u' hex hex hex hex
                 | '\u{' hex { hex } '}'
hex              = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
                 | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'
                 | 'a' | 'b' | 'c' | 'd' | 'e' | 'f'

(* ── Tagged values ──────────────────────────────────── *)
tagged-value    = qualified-ident quoted-string
qualified-ident = identifier { '.' identifier }

(* ── Identifiers ────────────────────────────────────── *)
identifier      = id-start { id-continue }
id-start        = '_' | ? char::is_alphabetic() ?
id-continue     = id-start | '-' | ? char::is_alphanumeric() ?

(* ── Indentation ────────────────────────────────────── *)
pipe            = '|'
pipes           = { pipe }

(* ── Comments (discarded) ───────────────────────────── *)
comment         = line-comment | block-comment
line-comment    = '#' { char } newline
block-comment   = '/*' { char | block-comment } '*/'

(* ── Terminals ──────────────────────────────────────── *)
newline         = '\n'
blank-line      = newline
char            = ? any UTF-8 code point except newline and unescaped '"' ?
```

> **Note on bare values:** The grammar above shows `value` in positions following `=` and `-`. The actual token consumed from the source may be `Str`, `Number`, `Bool`, `Null`, or `Tagged`. The bare-value inference mechanism described in §2.6 handles the mapping from raw source characters to typed tokens; it is a lexer-level concern and does not alter the grammar structure.

---

## Appendix A. Complete Example

The following document demonstrates all Spine features in combination:

```spine
# Spine full feature showcase

# ── Scalar key–value pairs ───────────────────────────────
app
| name    = Spine Showcase System
| version = 0.1.0
| mode    = production

# ── Nested objects (3+ levels deep) ─────────────────────
system
| runtime
| | env      = production
| | region   = eu-central-1
| | timezone = UTC
|
| limits
| | cpu    = 8
| | memory = 16GB           # bare string (does not parse as number)
| | io     = high
|
| logging
| | level  = info
| | format = json
| | sinks
| | | - stdout              # scalar array element
| | | - file
| | | | path     = /var/log/spine.log
| | | | rotation = daily
|
| telemetry
| | enabled  = true
| | exporter = otel
| | endpoint = https://telemetry.local

# ── Array of objects ─────────────────────────────────────
database
| replicas
| | -
| | | host = db.replica-1.local
| | | lag  = low
| |
| | -
| | | host = db.replica-2.local
| | | lag  = medium

features
| flags
| | -
| | | name    = new-ui
| | | enabled = true
| | | rollout = 0.5
| | -
| | | name    = dark-mode
| | | enabled = true
| | | rollout = 1.0

# ── Tagged literals ───────────────────────────────────────
meta
| created = std.date"2026-05-26"
| hash    = base64"c3BpbmUtZnVsbC1leGFtcGxl"

# ── Append (~) ────────────────────────────────────────────
| ~events
| | type      = created
| | timestamp = 2026-05-26T00:00:00Z
| ~events
| | type      = deployed
| | timestamp = 2026-05-26T18:00:00Z

# ── Dotted-path append ────────────────────────────────────
server
| host = localhost
~server.users
| name = alice
~server.users
| name = bob

# ── Multi-line string ─────────────────────────────────────
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

The following ill-formed documents illustrate each error category. A conforming parser MUST reject each with an error of the kind indicated.

```spine
# type-conflict: scalar then object
server = localhost           # defines server as String
server                       # attempts to reopen server as Object
| port = 8080                # error: type-conflict

# duplicate-key
host = localhost
host = example.com           # error: duplicate-key

# type-conflict via append: target is not an array
mode = production            # defines mode as String
~mode                        # attempts to append to mode
| env = staging              # error: type-conflict

# duplicate-key within a nested object
database
| host = db.local
| host = db.remote           # error: duplicate-key
```

---

## Appendix C. Known Gaps and Implementation Notes

This appendix is informational and is not part of the normative specification.

### C.1 Negative Numbers in Key Position

A token such as `-42` appearing at the start of a line — that is, not preceded by `=` or a parent `-` — is lexed as an array dash followed by the number `42`. A negative number literal at root level is not meaningful in the current grammar. This is intentional: Spine has no expression syntax that would require a signed numeric key.
