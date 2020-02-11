# Foolang IRs

``` mermaid
graph LR;
    mg1[MG];
    mg2[MG];
    Source-->|parse|ST-->|analyze|mg1-->|optimize|mg2-->|lower|LR;
```

## ST, Syntax Tree

Concrete representation of the parsed program.

Goals:
- Generated in a single pass from source.
- Accurate source locations.
- Interpretable: the bootstrap evaluator operates on this.
- Enough information to pretty-print the program using same newlines and
  syntax sugar, without access to original source.
- Platform independent

Current `Expr` in `parse.rs` isn't quite this, but will be. 

## MG, Message Graph

Abstract representation of the program in message send form.

Goals:
- Initially Generated in a single pass from FST, but can be further processed.
- Accurate source locations.
- Accurate dependency information: when current FMT depends on known 
  definitions of other classes, this dependency is is known so that
  changes can trigger deopt & reopt.
- Accurate variable names for source variables.
- Variable references and types fully resolved.
- Interpretable: the self-evaluator operates on this, probably.
- Suitable for partial evalution to resolve sends to known classes,
  eliminate unnecessary typechecks, inline blocks, etc. 
- Pretty printable in a form that makes it easy to understand what optimization
  has taken place:
  ``` foolang
  { |x::I64 y::Double| x+y } --> { |x::I64 y::Double| -> Double
                             -->   Double __add: y to: (Int __toDouble: x) } 
  ```
  Note: __foo message which parser should not accept, denoting unsafe primitive
  operations which don't typecheck their arguments that cannot be entered by
  user, but can result from program transformations.
- Platform independent

## LR, Linear Representation, Low-level Representation

Goals:
- Suitable for VM execution or translation to external low level languages
  like LLVM IR, C, etc.
