# Booleans

Blocks take `and:` and `or:` messages, including chains like:
`and:and:`and `or:or:` upto some reasonable length like three.

Operators `&` and `|` translate into `and:` and `or:` respectively,
with the normal precedence.

Thus blocks can be used to express short circuiting in a nice flat
way:

    {x test} and: {y test} and: {z test}

    {x test} or: {y test} or: {z test}

    {x test} & {y test} & {z test}

    {x test} | {y test} | {z test}

    {1 <= index} & {index <= size} | {allowOutOfBounds}

Booleans do not take these messages because then it would be possible to
say

    a < b and: (foo from: a to: b)

and assume the short-circuiting behaviour happens.

I assume that in classic Smalltalk environments being able to do
operations on booleans was a win because there was no need to
construct a block.

I also assume that having compiler optimize the block out should not
be hard.
