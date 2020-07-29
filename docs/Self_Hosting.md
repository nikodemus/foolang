# Self Hosting Plan

1. Bootstrap evaluator (done).
2. Self-hosted parser (done, mostly).
3. Self-hosted AST interpreter (done, mostly).
4. Self-hosted AST->C transpiler (WIP).
5. Self-hosted AST typechecker & method linker.
6. Self-hosted AST partial evaluator.

This should give a self hosted implementation with something resembling useful
performance.

AST might turn into just into just a bit more than AST, but the idea is to run
the core optimizations on a representation where there isn't much bookkeeping
going on.
