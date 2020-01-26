# Foolang Booleans

**Status**: I like this design, it does not match the current implementation.

Blocks take `and:` and `or:` messages, including chains like: `and:and:`and
`or:or:` upto some reasonable length like three or four.

Operators `&&` and `||` translate into `and:` and `or:` respectively,
with the normal precedence.

Thus blocks can be used to express short circuiting in a nice flat
way:

    {x test} and: {y test} and: {z test}

    {x test} or: {y test} or: {z test}

    {x test} && {y test} && {z test}

    {x test} || {y test} || {z test}

    {1 <= index} && {index <= size} || {allowOutOfBounds}

Booleans probably should not take these messages because then it would be
possible to say

    a < b and: (foo from: a to: b)

and assume the short-circuiting behaviour happens. Clarity first.

I assume that in classic Smalltalk environments being able to do operations on
booleans was a win because there was no need to construct a block.

However, since these are literal blocks then even an interpreter can trivially
optimiza the block away, replacing the send with `Primitive boolean:andBlock`
or similar.


