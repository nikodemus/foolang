# Enum

Supervisors OneForOne constant made me think.

Constants are better as flags than literals Because
then the compiler can do checking.

...but

```
@constant Foo = Opaque name: "Foo"
```

gets old fast.

What I want is something like:

```
@enum TypeName
   name1
   name2
   name3

# So this works
TypeName name1
```

OR

```
@enum TypeName
   Name1
   Name2
   Name3

# So this works
Name1
```

The classic C-use cases aren't pointless either.

```
@enum Typename
    name1: 0
    name2: 1
    name3: 3
```

No reasons methods cannot be defined on enums.
