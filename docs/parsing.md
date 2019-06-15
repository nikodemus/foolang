# Parsing Foolang

The initial bootstrap parser was implemented using LALRPOP in Rust.

I'm not too happy with the error reporting, though, and since the
grammar _should_ be LL(1) LALRPOP is overkill -- and to be frank,
I've always found LR parsers confusing.

I think I want to write a separate tokenizer then an OPG
on top of that.
