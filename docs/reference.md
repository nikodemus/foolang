# Reference

First a tokenizer is run on the input stream.

Token classes are:

   Identifier
   Sigil

A top-down Pratt-style precedence parser is run on the token stream.

The builtin parser has rules for:

- @class, @classMethod, @method, @constant, @prefix-parser, @infix-parser
- keyword messages
- unary messages
- let
- blocks
- arrays

@prefix-parser -
   with: { parser | parser parseExpression send: #neg }

@prefix-parser (
   with: { parser |

   }

@infix-parser +
   precedence: 100
   with: { left, parser |
     left send: #add with: [parser parseExpression]
   }

@infix-parser --
   precedence: 1
   with: { left, parser |
     left
   }

@infix-syntax 60 left + right
    left add: right

@prefix-syntax 100 - right
    right neg

@prefix-syntax 10 if

The initial bootstrap parser was implemented using LALRPOP in Rust.

I'm not too happy with the error reporting, though, and since the
grammar _should_ be LL(1) LALRPOP is overkill -- and to be frank,
I've always found LR parsers confusing.

I think I want to write a separate tokenizer then an OPG
on top of that.
