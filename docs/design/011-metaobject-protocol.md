# Metaobject Protocol

**Status**: WIP (design in progress, not implemented)

**Identifier**: 011-metaobject-protocol

**References**:
- none

**Prior Art**:
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
- 2020-08-11: initial incomplete version by Nikodemus

## Problem Description

How are methods resolved? (If a class inherits two interfaces with the same
selector, which one gets used?)

Is there a `super`, or similar way to send a message to an overridden
interface methods?

How to build classes programmatically?

What are the metaobjects that describe these issues?

How to avoid breaking encapsulation so that authority of the system object isn't
subverted by stealing authorizing objects through introspection or intercession?

How can the self-hosted interpreter access slots of the classes it creates?

## Proposal

A shallow metaobject protocol, without providing much in the way of readymade
building blocks.

Control access to metalevel objects through `Reflection` class and _Mirror_
instances.

The `Reflection` class provides compiler visibility into which objects are
reflected.

The `Mirror` interface provides authority to break encapsulation, as the general
purpose instance can only be obtained through `System#mirror`. Those
programmatically building classes (like the self-hosted interpreter) already
have access to their metaobjects, and as such can implement mirrors for them
without access to the system object.

---

### interface Protocol

Common ancestor of classes `Interface` and `Class`. 

---

### class Class

is Protocol

#### _direct method_ `name:` name `behavior:` [Behavior](#interface-behavior) `instanceBehavior:` [Behavior](#interface-behavior) `instanceLayout:` [Layout](#class-layout) -> [Class](#class-class)

Constructs a new instance of `Class`, with the specified behaviors and instances
layout.

---

### class Interface

is Protocol

#### _direct method_ `name:` name `behavior:` [Behavior](#interface-behavior) `instanceBehavior:` [Behavior](#interface-behavior) -> [Interface](#class-interface)

Constructs a new instance of `Interface`, with the specified behaviors.

---

### class Reflection

#### _direct method_ `of:` Object `in:` [Mirror](#interface-mirror) -> [Reflection](#class-reflection)

Provides access to a reflection, through which various metaobjects can be
accessed.

Send `#show:of:` to the mirror with a new empty reflection and the object
to be reflected as arguments.

Verifies that the reflection is no longer empty, and returns it.

Sketch:
``` foolang
let reflection = self _new.
mirror show: reflection of: object.
{ reflection _isEmpty } assertFalse.
reflection!
```

#### _method_ `isClassReflection` -> Boolean

Returns true if the reflected object is a `Class`.

#### _method_ `isInterfaceReflection` -> Boolean

Returns true if the reflected object is an `Interface`.

#### _method_ `isProtocolReflection` -> Boolean

Returns true if the reflected object is a `Protocol`.

#### _method_ `behavior` -> [Behavior](#class-behavior)

Provides access to the behavior of the reflected object. Raises an error
if the reflection is empty.

#### _method_ `layout` -> [Layout](#interface-layout)

Provides access to the layout of the reflected object. Raises an error
if the reflection is empty.

#### _method_ `instanceBehavior` -> [Behavior](#class-behavior)

Provides access to the behavior metaobject describing instances of the
reflected object. Raises an error if the reflection is empty, or if the
reflected object is not a `Protocol`.

#### _method_ `instanceLayout` -> [Behavior](#class-behavior)

Provides access to the layout metaobject of instances of the reflected object.
Raises an error if the reflection is empty, or if the reflected object is not a
`Class`.

#### _method_ `behavior:` [Behavior](#class-behavior) `layout:` [Layout](#interface-layout) -> None

Sets the behavior and layout of the reflection of a non-protocol instance.
Raises an error if the reflection is not empty. The reflection is no longer
empty after its behavior and layout have been set.

For use by those implementing their own mirrors.

#### _method_ `behavior:` [Behavior](#class-behavior) instanceBehavior: [Behavior](#class-behavior) `layout:` [Layout](#interface-layout) -> None

Sets the behavior, instance behavior, and layout of the reflection of an
Interface instance. Raises an error if the reflection is not empty. The
reflection is no longer empty after its behavior and layout have been set.

For use by those implementing their own mirrors.

#### _method_ `behavior:` [Behavior](#class-behavior) instanceBehavior: [Behavior](#class-behavior) `layout:` [Layout](#interface-layout) `instanceLayout:` [Layout](#interface-layout) -> None

Sets the behavior, instance behavior, layout, and instance layout of the
reflection of a Class instance. Raises an error if the reflection is not empty.
The reflection is no longer empty after its behavior and layout have been set.

For use by those implementing their own mirrors.

---

### interface Mirror

#### _direct method_ `show:` [Reflection](#class-reflection)  `of:` Object -> None

Sets the behaviors and layouts of the reflection to those of the object's
class.

**NOTE**: Only way to have access to an empty reflection is through `Reflection
of:in:`, which sends it to a `Mirror` as part of this message, and later checks
that it is no longer empty. The general mirror provided by `System#mirror`
requires an empty `Reflection`, allowing the compiler to reason about reflection
of standard classes in most cases, hopefully. More importantly it stops
unauthorized access to metalevel.

!> Above claim needs proof.

---

### class Behavior

Examples:
```
-- methods on integer 42
(Reflection of: 42 in: system mirror)
    behavior methodDictionary

-- methods on all integers (same method dictionary as result as above)
(Reflection of: Integer in: system mirror)
    instanceBehavior methodDictionary

-- direct method on Number interface 
(Reflection of: Number in: system mirror)
   behavior methodDictionary
   
-- classes and interfaces directly saying `is Number`
(Reflection of: Number in: system mirror)
   behavior immediateImplementors

-- classes implementing Number
(Reflection of: Number in: system mirror)
   behavior implementors select: { |impl| Class includes: impl }
```

#### _method_ `host` -> Procotol

Returns the class or interface whose behavior the receiver is.

#### _method_ `immediateBehaviors` -> Array of: Behavior

Returns an array of behaviors immediate to the receiver, ie. not inherited.

This corresponds to `is` declarations in class and interface definitions,
and appears in the same order as those do.

#### _method_ `immediateMethodDictionary` -> Dictionary from: Selector to: [Method](#interface-method)

Returns a dictionary which maps selectors to methods immediate to this
behavior, ie. not inherited.

#### _method_ `immediateImplementors` -> Array of: Protocol

Returns an array of all protocols which immediately implement this behavior, ie.
not through inherirance. This does not include `#host`.

**NOTE**: This returns regular class and interface objects, not behaviors!

#### _method_ `behaviours` Array of: Protocol

Returns a [C3 linearization](https://en.wikipedia.org/wiki/C3_linearization) of
all behaviors implemented by receiver, including inherited ones. This includes
the receiver as the first element.

#### _method_ `methodDictionary` -> Dictionary from: Selector to: [Method](#interface-method)

Returns a dictionary of methods available to instances of `#host`, including
inherited ones.

Construction is done based on the `#behaviors` linearization.

#### _method_ `implementors` -> Array of: Protocols

Returns an array of all protocols implementing this behavior either immediately
or through inheritance. This always includes `#host`.

**NOTE**: This returns regular class and interface objects, not behaviors!

---

### interface Layout

#### _method_ `host` -> Class

Returns the class whose layout this is. (Interfaces do not have layouts, only
behavior.)

#### _method_ `slots` -> Array of: [Slot](#interface-slot)

Returns and array of slots in the same order as they appeared in
the class definition.

#### _method_ `allocate:` initialValues -> Any

Returns an instance of `#host` class with given initial values.

---

### interface Slot

#### _method_ `name` -> Selector

Returns the name of this slot.

#### _method_ `type` -> Type

Returns the type constraint of this slot.

#### _method_ `read:` instance -> Any

Returns the value of this slot in _instance_. The returned value is of `#type`.

#### _method_ `write:` value `to:` instance -> Any

Writes _value_ to this slot in _instance_. Returns _value_.

---

### interface Method

#### _method_ `selector` -> Selector

#### _method_ `host` -> Protocol

Returns the class or interface in which this method is defined.

#### _method_ `argumentTypes` -> Array of: Type

Returns the argument types required by the method.

#### _method_ `returnType` -> Type

Returns the return type of the method.

#### _method_ `invokeOn:` receiver `with:` arguments -> Any 

Invokes the method using _receiver_ and _arguments_. Returns a value
consistent with `#returnType`.

---

## Summary

- _"How are methods resolved?"_ Using C3 linearization.
- _"Is there a `super` or equivalent?"_ Not answered yet.
- _"How to build classes and interfaces programmatically?"_ Not answered yet.
- _"What are the metaobjects that describe these issues?"_ Partial answer.
- _"How to avoid breaking encapsulation?"_ Answer: by requiring `System#mirror`
  to gain access to reflection.
- _"How can the self-hosted interpreter access slots of the classes it
  creates?"_ Answer: it has access to the metaobjects since it constructed
  the class.

#### Safety

Positive impact: currently `#__slotAt:` and such are used by the self-hosted
interpreter, breaking encapsulation.

#### Ergonomics

Neutral impact: doesn't matter for vast majority of code.

#### Performance

Neutral impact: compiler should be able to deal with this.

#### Uniformity

Positive impact: `class` and `interface` syntaxes could be replaced by
`define` syntax if one really wanted.

#### Implementation

Mixed. A metaobject protocol in general makes a lot of things easier and
more consistent, but there is some up-front work to be done. It remains to
be seen how much of this can be elided in the bootstrap evaluator.

#### Users

No users, no impact.

## Alternatives

- Not using mirrors, but making classes and interfaces direct metaobjects.
  Breaks encapsulation.
- Not having a metaobject protocol. Sucks.
- Having a different metaobject protocol. ...maybe, let's try this one first.

## Implementation Notes

None.

## Discussion

None.
