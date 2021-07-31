# WASM Support

**Status**: ADOPTED (not implemented)

**Identifier**: 015-wasm-support

**References**:
- https://emscripten.org/index.html
- https://surma.dev/things/c-to-webassembly/index.html
- https://surma.dev/things/raw-wasm/

**History**:
- 2021-07-30: initial version by Nikodemus

## Problem Description

I want Foolang to support WASM as a first-class backend, being able to easily
port Foolang program to run in browser.

The first use case is being able write a web REPL in Foolang. I'm focusing on
this on its own, with the assumption that it represents a larger class of similar
problems&mdash;without going speculation about how it does or doesn't match more
common needs.

Implementation sketch:

```foolang
class Main { document input }
     -- #tryParse: and #translateSyntax: elided

    direct method run: command in: system
        (self document: system javascript document
              input: StringOutput new)
            run!

    method run
        document body innerHTML := "<pre id=\"terminal\"></pre>".
        let terminal = document getElementById: "terminal".
        document
            addEventListener: "keypress"
            with: { |keypress|
                    let string = (Character code: keypress charCode) string.
                    terminal innerHTML += string.
                    input print: string.
                    string == "\n"
                        ifTrue: { self tryEval } }!

    method tryEval
        let syntax = self tryParse: input copyContent.
        syntax is False
            ifTrue: { return False }.
        input clear.
        let result = False.
        syntax do: { |each|
                     let ast = self translateSyntax: each.
                     result = ast evalIn: env }.
        terminal innerHTML += result displayString.
        terminal innerHTML += "\n> "!
end
```

The general pattern here is using the browser mostly as a presentation
layer for a larger body of Foolang code.

List of facilities required:

- Access to `document` and its properties. The required scope of access is
  fairly narrow, though. (Would be feasible to implement all by hand.)

- Access to `Keypress` propertires. The required scope of access is fairly
  narrow, though. (Would be feasible to implement all by hand.)

- Passing strings to Javascript.

- Passing blocks to Javascript and receiving callbacks.

### Decision Drivers

Quick experiment shows that Foolang-generated C-code compiles to WASM with
Emscripten just fine.

For web applications the control flow is very different from eg. console
ones, though: event driven instead of blocking. This inversion of control
requires hooking into Javascript APIs which is possible with Emcscripten,
but the C-apis don't look look really appetizing for Foolang use.

Emscripten is also an additional dependency, and while it implements a big chunk
of POSIX I don't really need that much, and Foolang already provides a mechanism
for abstracting over such things via System.

## Proposal

### System Libc vs WASM

Remove direct dependencies on libc from `main.c` and built-in classes.

Instead move those dependencies to `system_libc.c`, and provide equivalent ones
in `system_wasm.c`. (`malloc/free` at least: if there is no `System#files` or
`System#output` in WASM-land, then the `FILE*` API can be just stubs.)

### To Javascript and Back

Add `System#javascript`, returning an object which allows Foolang arbitrary to
access to the Javacript namespace.

Sending messages to `Javascript#document` supports strings, integers, floats,
and blocks as arguments, and arbitrary javascript objects as return values.

Lifetime of blocks passed to Javascript is extended to duration of the program:
Foolang pushed them onto a global array visible to GC, and they are passed to
Javascript as indexes into this array. The Javascript side code wraps them into
proxy functions which do the calling back to Foolang side.

Lifetime of returned non-primitive objects returned from Javascript to Foolang
is extended to the duration of the program: Javascript pushes them to a global
array visible to GC, and they are passed to Foolang as indexes into this array.
The Foolang side code wraps them into proxy objects which can be send arbitrary
messages to.

Lifetime of non-primitive objects passed as arguments to Foolang blocks from
Javascript is for the duration of the block: they are considered expired after
the block has returned. They are identified by an integer encoding the number of
the call and the argument index. Sending a message to on expired Javascript
objects from Foolang will cause an error.

Multipart keyword messages send to Javascript use the first part as the
Javascript method name, remaining being `with:`.

If Foolang does not yet support selectors like `name:=` or `name+=`, they
can be replaced with `set:to:` and `to:add:` methods.

```foolang
document body set: #innerHTML to: "Ok".
document body to: #innerHTML add: "...".
```

### Generate WASM with Clang, Not Emscripten

One dependency less, easier for me, for CI, for future users.

### Summary

Putting focus on Foolang side rather than Javascript side seems like the correct
choice.

The amount of extra bookkeeping seems fairly small, and should perform
acceptably for the domain.

The lifetime restrictions are irksome, but avoiding them would require adding
finalizers, and Foolang doesn't have finalizers yet -- and I would worry about
the performance.

#### Safety

No impact.

#### Ergonomics

No impact.

#### Performance

No impact.

#### Uniformity

Positive impact: instead of having to write magical glue code in C or understand
Foolang name mangling the C layer remains invisible.

#### Implementation

Negative impact: the keeping of additional roots and having proxies on both sides
is extra work - not to mention the changes to `main.c`.

#### Users

No users, no impact.

## Alternatives

- Wrangling with Emscripten. It does a _lot_, and has a lot of functionality
  I'm unlikely to add in a hurry. However, giving that the proxy objects pass
  along arbitrary messages a lot things should be still doable.

- Focusing more on the Javascript side: ie. how to write code in Foolang that
  can be used from Javascript. I don't think that's actually in conflict with
  what I'm doing here, though.

## Implementation Notes

No implementation yet.

## Discussion

No discussion.
