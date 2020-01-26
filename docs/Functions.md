# Foolang Functions

I suspect functions are a good thing.

Not just to duplicate mathematical syntax, but to
express things: having a class that is never instantiated
just to have a place for a single class method seems wrong.

You could already fake them with blocks anyhow:

```
define Fibonacci = { :x | ... }
```

Converting `a + b` to a generic function call add(a, b)
is easier to optimize and faster unoptimized then double-dispatch, I think.

Restricting names to Capitalized words seems like a small price
to pay.

```
function Fibonacci(x)
    if x < 2 {
        1
    } else {
        Fibonacci(x-1) + Fibonacci(x-2)
    }
end
```

At the same time, making function call syntax alias for block value
messages appeals:

```
foo(x, y, z) === foo value: x value: y value: z
```
