# Reflection in Foolang

Foolang uses mirrors for reflection.

Class objects are first class objects: they can be passed around
as values, etc. Because class objects are global they are also immutable.

They are not a reflective tool, however: you can only send specific messages
to them, and those messages don't expose any reflective facilities.

Specifically, you cannot ask a class for its name, its slots,
or its methods. To do this you need a _mirror_ on the class.

    Mirror reflect: class

This makes it trivial for the compiler to tell if the program uses
reflection or not: if Mirror class is not linked in, it is not needed
-- and even if it is needed we have a good chance of being able to
prove that it can only reflect on instances of certain classes.
