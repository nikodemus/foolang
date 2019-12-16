# Array

Basic approach: steal from Julia, numpy, and Fortress

Array is an abstract interface. Array shape and size may be mutable.
Users can implement new types of arrays.

Builtin array types are Vector, Matrix, and NdArray. Builtin types use
[] syntax as the constructor.

NOTE: Elements of array constructor that evaluate to arrays are
flattened into the newly created array.

NOTE: Elements of array constructor expressions must be parenthesized
unless they are constants, variable references, or prefix messages to
either constants or variable references.

XXX: This may bear reconsideration: this allows using whitespace to
catenate inside arrays, which is kind of nice... but we could just as
well use eg. ellipsis to do that. (It also makes commas optional, which
is a mixed bag.) The syntax is largely stolen from Fortress.

## Specialization

Arrays are specialized using a specialization method:

    [1, 2, 3] of: U8

Once things move further along, something like

    [::U8 1, 2, 3]

may be used instead.

XXX: Once I have allocation types for arrays, they will be denoted using a type parameter:

    Vector[F64]
    Matrix[U8]

## Indexing & Broadcasting

Indexes start at 1. Rationale: this makes

    1 to: 10

easy to understand and interacts nicely with slicing.

n elements from start:

         array[n from: start]

from start to end

         array[start to: end]

Negative indexes index from the end.

`array[x]` is sugar for array at: x

`array[x] = y` is sugar for array at: x put: y

Both of these go though double-dispatch on the index:

       Array method at: pos
          pos atArray: self
       Integer method atArray: array
          array atIndex: self
       Array method atArray: array
          array atIndexes: self
       Interval method atArray: array
          array atInterval: self

The corresponding put methods will broadcast as necessary.

## Vector

1-dimensional built-in Array type. Mutable in content and length. For
purposes of matrix operations vectors are considered to be column
vectors.

Inside [] comma catenates elements of a vector. Trailing comma is allowed.

    -- 1-element vector
    [1]

    -- 3-element vector
    [1, 2, 3]

    -- using parenthesis to send messages
    [(origin x), (origin y), 0.0]

    -- 6-element vector: the elements of 'a' are catenated by the comma
    let a = [1, 2]
    [a, 3, 4] --> [1, 2, 3, 4]

## Matrix

2-dimensional built-in Array interface. Mutable in content and shape.
Storage is column major.

Inside [] space catenates horizontally, whereas semicolon and newline catenate
vertically.

    -- 3x1 column matrix
    [1; 2; 3]

    -- 1x3 row matrix
    [1 2 3]

    -- 2x3 matrix
    [1 2 3; 4 5 6]

    [1 2 3
     4 5 6]

    -- 4x3 matrix using the flattening
    let rows = [1 2 3; 4 5 6]
    [rows
     rows]

    -- 2x6 matrix using the flattening
    let rows = [1 2 3; 4 5 6]
    [rows rows]

Matrices support the usual operations.

## NdArray

N-dimensional built-in Array type. Mutable in content and shape.

Sequences of semicolons catenate along the Nth dimension, where N is
number of semicolons plus one.

    -- 3x3x2x2 ndarray
    [1 0 0
     0 0 0
     0 0 0 ;; 0 1 0
              0 0 0
              0 0 0 ;;;
     0 0 1
     0 0 0
     0 0 0 ;; 0 0 0
              1 0 0
              0 0 0]
