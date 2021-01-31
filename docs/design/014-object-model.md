# Object Model

**Status**: WIP (partially implemented in transpiler)

**Identifier**: 014-object-model

**References**:
- Supersedes the following design notes:
  - [002 - No Class Inheritance](design/old/002-no-class-inheritance.md)
  - [011 - Metaobject Protocol](design/old/011-metaobject-protocol.md)
- [Metaclasses are First Class: the ObjVLisp
  Model](http://stephane.ducasse.free.fr/Web/ArchivedLectures/p156-cointe.pdf)
  by Pierre Cointe.
- The Art of the Metaobject Protocol, by Gregor Kiczales, Jim des Rivieres, and
  Daniel G. Bobrow.
- [Traits: Composable Units of
  Behaviour](http://scg.unibe.ch/archive/papers/Scha03aTraits.pdf)
- [Efficient Multimethods in a Single Dispatch
  Language](http://www.laputan.org/reflection/Foote-Johnson-Noble-ECOOP-2005.pdf)
- [Flexible Object Layouts: Enabling Lightweight Language Extensions by
  Intercepting Slot
  Access](https://rmod.inria.fr/archives/papers/Verw11a-OOSPLA11-FlexibleObjectLayouts.pdf)
- [C3 linerarization](https://en.wikipedia.org/wiki/C3_linearization)

**History**:
- 2021-01-23: initial version by Nikodemus
- 2021-01-31: adding layouts, explicit instantiation description, description of
  interface implementation in terms of classes, only one kind of subtyping.

## Problem Description

What is the Foolang object model?

Consider the following task:

> At runtime, create a new class such that the class instance responds to a
specified set of methods, can be instantiated, and the that the instances
respond to another specified set of methods and have their own data.

### Primary Decision Drivers

- Consistency & uniformity
- Performance
- Allows runtime class allocation
- Allows later development of a more complete metaobject protocol

### Secondary Decision Drivers
- Ease of understanding
- Ease of implementation

## Proposal

![Figure 1. Instantiation and subtyping relationships](014-object-model-figure-1.png)
Figure 1. Illustration of parallel instantiation and subtyping relationships.

The following object model is heavily influenced by ObjVLisp.

1. An object represents a set of behaviours (_methods_) associated with some
   data.

2. Every object belongs to a _class_, which specifies its methods and data
   representation (_layout_), and allows creation of new objects belonging to
   the said class (_instances_). Instances of a class share the same layout and
   behaviour, but differ in data content. Classes are always immutable.

3. The only protocol for invoking an object's behaviour is message passing. On
   receipt of a message the class of the receiving object is responsible for
   determining which method correspond to the said message and its arguments,
   and invoking it.

4. Classes are objects as well, and are therefore instances of other classes.
   Classes of classes are called metaclasses. The root metaclass of the system
   is `Class`, which is an instance of itself.

5. Classes which specify only methods with no layout are called _abstract
   classes_. Non-abstract classes may be referred to as concrete classes for
   clarity.

6. Classes can subtype each other through _inheritance_.

   A class can inherit from at most one concrete class, and from an unlimited
   number of abstract classes.

   If a class inherits from a concrete class, it inherits both the layout and
   all methods of the inherited class, and can only extend the inherited class
   by adding new methods; overriding existing methods or changing the layout is
   not allowed.

   If a class inherits from an abstract class, it inherits all methods of the
   inherited class, and can specify a layout, extend the inherited class by
   adding new methods, as well as override existing methods as long as declared
   argument types are invariant and return-types are covariant.

   The inheritance tree is rooted in the abstract class `Any`, inherited by
   all classes, representing the most common behaviour shared by all objects.

8. Interfaces do not exists as concepts at object model level, but are a
   language level concept implemented on top of classes.

   For each interface there are three associated classes:

   - An abstract direct interface class , providing _direct_ methods
     of the interface.

   - A concrete interface metaclass, inheriting from the direct interface class,
     providing the _own_ methods of the interface.

   - An abstract instance interface class, instance of the interface metaclass,
     providing _instance_ methods of the interface. This object is called
     _"the interface object"_ at the language level.

   Type _T_ inheriting from interface _I_ at the language level translates to the
   following at level of the object model:

   - Class _T_ inherits from the abstract instance interface class associated
     with _I_, instances of _T_ thereby gaining the instance methods of _I_.

   - Metaclass of class _T_, ie. _T class_ inherits from the abstract direct interface class associated with
     _I_, class _T_ thereby gaining the direct methods of _I_.

  See _Any_ and _Number_ in Figure 1.

?> Class immutability is absolutely required for classes known at compile time,
since they will be allocated in static space and shared between multiple threads
without locking. Class immutability could be relaxed for classes created at
runtime, but that would lead to an inconsistency between runtime created and
compile-time created classes that would be undesirable.

?> Class immutability in development mode / environment will be relaxed,
allowing the development environment to add methods to classes, change layouts,
etc. The details of this are to be defined later.

Pseudocode example of runtime instantiation:

```foolang
let mymetaclass
 = Class
       name: "a metaclass"
       extends: Class
       methods: [#test -> { |this| "the class" }].

let layout
 = Layout
       a: Integer
       b: Integer.

let myclass
 = mymetaclass
       name: "a class"
       layout: layout
       methods: [#test -> { |this|
                            "({layout get_a: this}, {layout get_b: this})" }].

let instance = myclass a: 1 b: 2.

mymetaclass name --> "a metaclass"
myclass name     --> "a class"
myinstance name  --! Does not understand

mymetaclass test --! Does not understand
myclass test     --> "the class"
myinstance test  --> "(1,2)"
```

### Summary

Proposal outlines a uniform object model that seems to cover the cases Foolang
is driving towards.

This specification implies that layout object and and instance methods must be
stored as part of the class. If we assume that this access is not public it does
not provide ad-hoc reflection, but allows implementation of mirrors as well as
allowing classes to provide selective reflection:

```foolang
class Demo {}
   direct method allMethods
       Self _allMethods!
   direct method foo
       42!
   method allMethods
       Self _allMethods!
   method bar
       42!
end

Demo allMethods --> [#allMethods, #foo]
Demo new allMethods --> [#allMethods, #bar]
```

#### Safety

None.

#### Ergonomics

Mixed impact.

On one hand this is a fairly simple and clean model without exceptional cases.

On the other hand the language level concept of _interface_ maps to multiple
classes, complicating the picture significantly - but users do not generally
have to worry about this.

#### Performance

Minor impact.

Metaclasses delegating instantiation to layouts stored in the class objects
is a small source of runtime overhead for when instantiating unknown classes.

#### Uniformity

Positive impact. Cleans up the mess I had before nicely.

#### Implementation

Minor impact. On the transpiler side most of the work is already done, and the
mess needed to be cleaned up anyhow.

#### Users

No users, no impact.

## Alternatives

- Having metaclasses specify the object representation instead of adding laouts.
- Making class abstract and adding builtin concrete subclasses specifying core
  layouts such as: StandardClass, ImmediateFloatClass, etc.
- Smalltalk model with an special Metaclass class.
- Having explicit interface objects (instead of abstract classes) in the model.

## Implementation Notes

Self hosted transpiler is on it's way to implementing this model. Bootstrap
implementation is different, and will likely remain that way until it gets
thrown out.

## Discussion

None.
