# foolang

## Parts

### Parser

Smalltalkish parser written in Rust.

### Bootstrap Compiler

Written in Rust.

Goal: specify VM core in foolang:

    "newContext: newMethod on: newSelf
        stack push: context
        context = Context new: newMethod on: newSelf with: context args"

    "invoke: method
        self newContext: method on: receiver"

- Generates code for methods given class description, method source,
  and backend.
- Initial backend is C.
- Supports:
  - self slot references and assignments
  - argument references
  - messages
  - constants
  - simple blocks (no closures)
- Quality of generated code unimportant, as long as it works.

### GC

Steal from Jazzlang, but give up on the double-wide stuff. Double-word
allocations.

Allocation header:
  Bits 00-01: GC marks
  Bits 02-09: number of raw words
  Bits 10-17: number of gc slots
  Bits 18-25: number of weak slots
  Bits 26-28: no tail / raw tail / weak tail / pointer tail
  Bits 29-31: n^2 = tail element width in bytes if raw tail

## Specificationish

Smalltalk syntax, except:

- Comma used for message chaining instead of parentheses.
- Blocks use {}
- Blocks can have implicit argument _.
- x => { ... } desugars into { ... } value: x

## Motivating Examples

### Example 1

Using blocks:

    Backend select: #postgres => {
      _ connect: "localhost" => {
          _ query: "select * from users", do: { "User: {_ name}" print }.
          _ query: "select * from suppliers", do: { "Vendor: {_ name}" print }.
      }.
      _ connect: "remote" => {
          _ query: "select * from users", do: { "User: {_ name}" print }.
          _ query: "select * from suppliers", do: { "Vendor: {_ name}" print }.
      }.
    }.

Factored using phrases:

    define -print-names-in-table: table as: pretty {
      _ query: "select * from {table}", do: { "{pretty}: {_ name}" print }
    }

    define -print-main-tables {
      _ -print-names-in-table: "users" as: "User".
      _ -print-names-in-table: "suppliers" as: "Vendor".
    }

    Backend select: #postgres => {
      _ connect: "localhost" -print-main-tables.
      _ connect: "remote" -print-main-tables.
    }.

## References

- Smalltalk-80: The Language and Its Implementation
  http://www.mirandabanda.org/bluebook/bluebook_imp_toc.html

## MMmmmaaaybe

- Serialize classes using json:

  { "class": "Class",
    "name": "Point",
    "slots": ["x", "y"],
    "methods": [{ "type": "method", "selector": "foo:with:",
                  "args": ["bar", "quux"],
                  "body": "bar ding: quux, reportTo: self"}]}

