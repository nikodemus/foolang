# Array Indexing

Sequence indexing, really.

Indexes start at 1.

Rationale: this makes

    1 to: 10

easy to understand and interacts nicely with slicing.

n elements from start:

         array[n from: start]

from start to end

         array[start to: end]

Negative indexes index from the end.

array[x] is sugar for array at: x

array[x] = y is sugar for array at: x put: y

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


