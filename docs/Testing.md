# Testing

It seems pretty obvious that property based testing ala hypothesis is the way to go.

## How Hypothesis Works

Test functions are annotated with generators:

    @given(ascii(), float())
    def test_foo(x, y):
        assert stuff(x, y)


Hypothesis remembers failing tests, but doesn't recommend distributing the database.
Ensuring specific examples are used is done with

    @example(y=-0.0)

annotation.

Hypothesis also tries to simplify the failing example.

Users can filter generated examples.

Plays nice with pytests discovery.

note() function to add information about a running test in case of failure.

event() similar but for statistics about what kind of cases where run.

assume() used to mark data as invalid (to avoid generating similar ones again)

target(obs) allows guiding generation to better stuff

Defaults:
- 200 attempts to generate an example
- 100 examples run at most

## Sketching how similar things could look in foolang

I don't have decorators/annotations like that, so explicit messages are needed.

    assert given: [AsciiStrings, Floats]
           that: {|s,f| ...}
           testing: "my important thing"

Explicit examples seem _really_ important.

    let examples = ["", "\n"].
    assert given: (Strings including: examples)
           that: {|s| s toString == s }

Passing them to generators 

I think this gets me going.



Generators are step n. 1.

Integer:
-1
0
1
-max
max
random

     -- instances keep track of things they've generated, avoiding dupes
     -- that can be either by linears strategy or maintaining history.
     --
     -- they're passes a Random instance to use as they wish
     let gen = generator new: random.
     




   

