# Foolang Arrays

**Basic approach**: steal from Julia, numpy, and Fortress

**Status**: interface design mostly there (small holes), implementation
not really even started.

`Array` is an abstract interface. Array shape and size may be mutable.
Users can implement new types of arrays.

Builtin array types are `Vector`, `Matrix`, and `NdArray`. Builtin types use
`[]` syntax as the constructor. (Or maybe `{}` -- the jury is still out.)

**NOTE**: In current design elements of array constructor expressions must be
parenthesized unless they are constants, variable references, or prefix messages
to either constants or variable references. This is to allow using whitespace as
separators in matrices and ndarrays.

## Specialization

Arrays are specialized using a specialization method:

    [1 2 3] of: U8

A special syntqax along the lines of

    U8[1, 2, 3]

is also an option, but seems less appealing right now.

**XXX**: Once I have allocation types for arrays, they will be denoted using a
type parameter, eg:

    Vector[F64]
    Matrix[U8]

This does _not_ necessarily imply general parameteric types -- but they
are needed for arrays.

## Indexing & Broadcasting

Indexes start at 1. Rationale: this makes

    1 to: 10

easy to understand and interacts nicely with slicing _methods_. Djikstra's points are well
made though, and I may well regret this.

n elements from start:

         array[n from: start]

from start to end

         array[start to: end]

Negative indexes index from the end.

`array[x]` is sugar for array at: x

`array[x] = y` is sugar for array at: x put: y

(The thing that most vexes me about 1-based indexing is that zero is not a valid
index. Still, a correctly predicted branch should be effectively free.)

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

Elements are separated by spaces.

Ellipsis flattens the previous expression into the vector.

    -- 1-element vector
    [1]

    -- 3-element vector
    [1 2 3]

    -- using parenthesis to send messages
    [(origin x) (origin y) 0.0]

    -- 6-element vector: the elements of 'a' are flattened by the ellipsis
    let a = [1, 2]
    [a... a...] --> [1 2 1 2]

    -- 10 element vector from an interval
    [(1 to: 5)...] -> [1 2 3 4 5]

## Matrix

2-dimensional built-in Array interface. Mutable in content and shape.
Storage is column major, probably.

Space separates elements of a row (dimension 2). Semicolon separates
rows (dimension 1). To create a matrix with just a single row add a
trailing semicolon. (Otherwise the result is a vector.)

Ellipsis flattens the previous expression into the matrix.

    -- 1x1 matrix
    [1 ;]

    -- 3x1 column matrix
    [1; 2; 3]

    -- 1x3 row matrix
    [1 2 3 ;]

    -- 2x3 matrix
    [1 2 3; 4 5 6]

    [1 2 3;
     4 5 6]

    -- 4x3 matrix using the flattening
    let rows = [1 2 3;
                4 5 6].
    [rows... ;
     rows...]

    -- 2x6 matrix using the flattening
    let rows = [1 2 3;
                4 5 6].
    [rows... rows...]

Matrices support the usual operations.

## NdArray

N-dimensional built-in Array type. Mutable in content and shape, including
dimensions.

Space separates elements of a row (dimension 2). Semicolon separates
rows (dimension 1). Sequences of N semicolons separate the (N+1)th
dimensions.

To create an ndarray with less than 3 dimensions add two trailing semicolons.
(Or rather: an ndarray where the final dimension has size 1.)

Ellipsis flattens the previous expression into the matrix.

    -- 1x3x1 ndarray
    [1 2 3 ;;]

    -- 2x2x2 ndarray
    [1 2;
     3 4 ;; 5 6;
            7 8]

    -- 3x3x2x2 ndarray using flattening
    let a = [1 0 0;
             0 1 0;
             0 0 1].
    [a... ;; a... ;;;
     a... ;; a...]

