XXX issues
- how to express type "this must be a protocol inheriting from x"
  (not an instance, but a protocol!)
- how to express type "constant x"

XXX compromises for convenience (printing, mainly!)
- #class method in Any
- #name method in Class and Interface

Use cases:
- reflection on interpreter objects producing an interpreter mirror
- interpreter being able to reflect on it's own objects without system object
- being able to get the host mirror on interpreter objects if you want it
  `system reflection of: anInterpreterObject in: Mirror` vs
  `system reflection of: anInterpreterObject in: ObjectMirror`

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

A simple metaobject protocol, without providing much in the way of readymade
building blocks.

Access to existing metalevel objects is through `Mirror` and `Reflection`
interfaces. (New metalevel objects can be created by baselevel code.)

The `Reflection` interface is the "raw" reflective API, the general version of
which is accessible through `System#reflection` method, thereby rooting
the authority to do reflection in the system object.

The `Mirror` interface in contrast functions as a way of making reflection
visible to the compiler, as direct interface references are _much_ easier
to reason about than messages and dataflow.

Code constructing classes programmatically already has access to the their
metaobjects as part of the construction, and can create mirrors for them without
access to the system object&mdash;and is able to create a `Reflection` class for
use with the constructed objects if so desired.

---

### Reflection (interface)

* **method** `reflectee` -> Object

  Returns the object being reflected.

* **method** `classReflection` -> [Reflection](#reflection-interface)

  Returns a reflection for the class of the object being reflected.

* **method** `behavior` -> [Behavior](#behavior-class)

  Returns the behavior of the reflected object.

* **method** `layout` -> [Layout](#layout-class)

  Returns the layout of the reflected object.

---

### SystemReflection (class)

is [Reflection](#reflection-interface)

* **direct method** `of:` Object `in:` MirrorInterface -> [Mirror](#mirror-interface)

  Returns a [Mirror](#mirror-interface) reflecting on _object_ using built-in
  reflection facilities.

  The _mirror interface_ must be a [Protocol](#protocol-interface) implementing
  [Mirror](#mirror-interface). Most common use case is to use the
  [Mirror](#mirror-interface) interface directly, but other implementations of
  [Mirror](#mirror-interface) may be used to control the mirror class selection
  (see below.)

  Sends `#mirrorClassFor:` message to the _mirror interface_ with the _object_,
  to obtain a mirror class.

  Sends `#reflection:` message to the mirror class, with a
  [SystemReflection](#systemreflection-class) of the _object_.

  This method is the only way to obtain a
  [SystemReflection](#systemreflection-class).

---

### Mirror (interface)

* **required direct method** `reflection:` [Reflection](#reflection-interface) -> [Mirror](#interface-mirror)

  Returns a [Mirror](#mirror-interface) with the given _reflection_.

* **direct method** `mirrorClassFor:` object -> [Class](#class-class)

  Returns a class inheriting [Mirror](#mirror-interface) suitable for use with
  _object_.

  Default implementation sends the message `#mirrorClassUsing:` to the _object_
  with the receiver. This allows eg. proxy classes to provide mirrors reflecting
  on the proxied object instead.

  Default implementations of `#mirrorClassUsing:` in `Object`, `Interface`, and
  `Class` send `#mirrorClassForObject`, `#mirrorClassForInterface`, and
  `#mirrorClassForClass` respectively back to the mirror interface they received.

  **NOTE**: Implementations of `mirrorClassUsing:` should not directly refer to
  classes implementing [Mirror](#mirror-interface), that creates unnecessary
  references to `Mirror` in them, causing compiler to think reflection may be
  happening whenever a class implements `mirrorClassUsing:`. This can be avoided
  using the `#mirrorClassFor*` sends.

* **direct method** `mirrorClassForObject` -> MirrorClass

  Returns `ObjectMirror`.

* **direct method** `mirrorClassForInterface` -> MirrorClass

  Returns `InterfaceMirror`.

* **direct method** `mirrorClassForClass` -> MirrorClass

  Returns `ClassMirror`.

* **method** `reflectee` -> Object

  Returns the object being reflected, aka the reflectee.

* **method** `classMirror` -> [ClassMirror](#classmirror-class)

  Returns a mirror for the class of the reflectee.

* **method** `behavior` -> [Behavior](#behavior-class)

  Returns the behavior of the reflectee.

* **method** `layout` -> [Layout](#layout-class)

  Returns the layout of the reflectee.

---

### class ObjectMirror

is Mirror

* **direct method** `mirrorClassFor`: Object -> MirrorClass

  Returns `ObjectMirror`.

---

### class ClassMirror

is Mirror

* **direct method** `mirrorClassFor`: Object -> MirrorClass

  Returns `ClassMirror`.

* **method** `name` -> Selector

  Returns the name of the class being reflected.

* **method** `instanceBehavior` -> [Behavior](#class-behavior)

  Returns the behavior of the instances of the class.

* **method** `instanceLayout` -> [Behavior](#class-behavior)

  Returns the layout of the instances of the class.

---

### class InterfaceMirror

is Mirror

* **direct method** `mirrorClassFor`: Object -> MirrorClass

  Returns InterfaceMirror.

* **method** `name` -> Selector

  Returns the name of the interface being reflected.

* **method** `instanceBehavior` -> [Behavior](#class-behavior)

  Returns the behavior of the instances of the class.

---

### interface Protocol

Common ancestor of classes `Interface` and `Class`.

---

### class Class

is Protocol

* **direct method** `name:` name `behavior:` [Behavior](#interface-behavior)
  `instanceBehavior:` [Behavior](#interface-behavior) `instanceLayout:`
  [Layout](#class-layout) -> [Class](#class-class)

   Constructs a new instance of `Class`, with the specified behaviors and instance
   layout.

---

### class Interface

is Protocol

* **direct method** `name:` name `behavior:` [Behavior](#interface-behavior) `instanceBehavior:` [Behavior](#interface-behavior) -> [Interface](#class-interface)

  Constructs a new instance of `Interface`, with the specified behaviors.

---

### class Behavior

Examples:
``` foolang
-- methods on integer 42
(system reflection of: 42 in: Mirror)
    behavior methodDictionary

-- methods on all integers (same method dictionary as result as above)
(system reflection of: Integer in: Mirror)
    instanceBehavior methodDictionary

-- direct methods on Number interface
(system reflection of: Number in: Mirror)
   behavior methodDictionary

-- all classes and interfaces directly saying `is Number`
(system reflection of: Number in: Mirror)
   behavior immediateImplementors

-- all classes implementing Number, directly or indirectly
(system reflection of: Number in: Mirror)
   behavior implementors select: { Class includes: _ }
```

* **method** `host` -> Procotol

Returns the class or interface whose behavior the receiver is.

* **method** `immediateBehaviors` -> Array of: Behavior

Returns an array of behaviors immediate to the receiver, ie. not inherited.

This corresponds to `is` declarations in class and interface definitions,
and appears in the same order as those do.

* **method** `immediateMethodDictionary` -> Dictionary from: Selector to: [Method](#interface-method)

Returns a dictionary which maps selectors to methods immediate to this
behavior, ie. not inherited.

* **method** `immediateImplementors` -> Array of: Protocol

Returns an array of all protocols which immediately implement this behavior, ie.
not through inherirance. This does not include `#host`.

**NOTE**: This returns regular class and interface objects, not behaviors!

* **method** `behaviours` Array of: Protocol

Returns a [C3 linearization](https://en.wikipedia.org/wiki/C3_linearization) of
all behaviors implemented by receiver, including inherited ones. This includes
the receiver as the first element.

* **method** `methodDictionary` -> Dictionary from: Selector to: [Method](#interface-method)

Returns a dictionary of methods available to instances of `#host`, including
inherited ones.

Construction is done based on the `#behaviors` linearization.

* **method** `implementors` -> Array of: Protocols

Returns an array of all protocols implementing this behavior either immediately
or through inheritance. This always includes `#host`.

**NOTE**: This returns regular class and interface objects, not behaviors!

---

### interface Layout

* **method** `host` -> Class

Returns the class whose layout this is. (Interfaces do not have layouts, only
behavior.)

* **method** `slots` -> Array of: [Slot](#interface-slot)

Returns and array of slots in the same order as they appeared in
the class definition.

* **method** `allocate:` initialValues -> Any

Returns an instance of `#host` class with given initial values.

---

### interface Slot

* **method** `name` -> Selector

Returns the name of this slot.

* **method** `type` -> Type

Returns the type constraint of this slot.

* **method** `read:` instance -> Any

Returns the value of this slot in _instance_. The returned value is of `#type`.

* **method** `write:` value `to:` instance -> Any

Writes _value_ to this slot in _instance_. Returns _value_.

---

### interface Method

* **method** `selector` -> Selector

* **method** `host` -> Protocol

Returns the class or interface in which this method is defined.

* **method** `argumentTypes` -> Array of: Type

Returns the argument types required by the method.

* **method** `returnType` -> Type

Returns the return type of the method.

* **method** `invokeOn:` receiver `with:` arguments -> Any

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
