# Functions

I suspect functions are a good thing.

Not just to duplicate mathematical syntax, but to
express things: having a class that is never instantiated
just to have a place for a single class method seems wrong.

You could already fake them with blocks anyhow:

```
@constant Fibonacci = { :x | ... }
```

Converting `a + b` to a generic function call add(a, b)
is easier to optimize and faster unoptimized then double-dispatch.

Restricting names to Capitalized words seems like a small price
to pay.

```
@function Fibonacci(x)
    if x < 2 {
        1
    } else {
        Fibonacci(x-1) + Fibonacci(x-2)
    }
```
