# Array

Basic approach: steal from Julia, numpy, and Fortress

Array is an abstract interface. Array shape and size may be mutable.
Users can implement new types of arrays.

Builtin array types are Vector, Matrix, and NdArray. Builtin types use
[] syntax as the constructor.

NOTE: Elements of array constructor expressions must be parenthesized
unless they are constants, variable references, or prefix messages to
either constants or variable references.

NOTE: Elements of array constructor that evaluate to arrays are
flattened into the newly created array.

XXX: Once things move further along, something like

    [::U8 1, 2, 3]

will be used to specify array allocation type. Default will be the
lowest common denominator that the compiler can derive.

XXX: Once I have allocation types for arrays, they will be denoted using a type parameter:

    Vector[F64]
    Matrix[U8]

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
