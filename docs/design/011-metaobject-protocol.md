XXX issues
- how to express type "this must be a protocol inheriting from x"
  (not an instance, but a protocol!)
- how to express type "constant x"

XXX compromises for printing
- #class method in Any
- #name method in Class and Interface

XXX decisions
- direct methods on classes and interfaces are instance methods on metaclasses 
- Class is an interface
- Interface is a interface

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

Is there a `super`, or similar way to send a message to an overridden interface
methods?

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

### Reflection

(an Interface)

#### Instance Methods

* `reflectee` -> Object

  Returns the object being reflected.

* `classReflection` -> [Reflection](#reflection)

  Returns a reflection for the class of the object being reflected.

* `behavior` -> [Behavior](#behavior)

  Returns the behavior of the reflected object.

* `layout` -> [Layout](#layout)

  Returns the layout of the reflected object.

---

### SystemReflection

(a Class)

#### Interfaces

- [Reflection](#reflection)

#### Direct Methods

* `of:` Object `in:` MirrorInterface -> [Mirror](#mirror)

  Returns a [Mirror](#mirror) reflecting on _object_ using built-in reflection
  facilities.

  The _mirror interface_ must be a [Protocol](#protocol) implementing
  [Mirror](#mirror). Most common use case is to use the
  [Mirror](#mirror) interface directly, but other implementations of
  [Mirror](#mirror) may be used to control the mirror class selection
  (see below.)

  Sends `#mirrorClassFor:` message to the _mirror interface_ with the _object_,
  to obtain a mirror class.

  Sends `#reflection:` message to the mirror class, with a
  [SystemReflection](#systemreflection) of the _object_.

  This method is the only way to obtain a [SystemReflection](#systemreflection)
  instance.

---

### Mirror

(an Interface)

#### Direct Methods

* **required** `#reflection:` [Reflection](#reflection) -> [Mirror](#mirror)

  Returns a [Mirror](#mirror) with the given _reflection_. This message
  is sent by [Reflection](#reflection)`#of:in:` with the newly created
  reflection to a specific mirror class.

* `mirrorClassFor:` object -> [Class](#class)

  Returns a class inheriting [Mirror](#mirror) suitable for use with
  _object_.

  Default implementation sends the message `#mirrorClassUsing:` to the _object_
  with the receiver. This allows eg. proxy classes to provide mirrors reflecting
  on the proxied object instead.

  Default implementations of `#mirrorClassUsing:` in `Object`, `Interface`, and
  `Class` send `#mirrorClassForObject`, `#mirrorClassForInterface`, and
  `#mirrorClassForClass` respectively back to the mirror interface they received.

  **NOTE**: Implementations of `mirrorClassUsing:` should not directly refer to
  classes implementing [Mirror](#mirror), that creates unnecessary
  references to `Mirror` in them, causing compiler to think reflection may be
  happening whenever a class implements `mirrorClassUsing:`. This can be avoided
  using the `#mirrorClassFor*` sends.

* `mirrorClassForObject` -> MirrorClass

  Returns `ObjectMirror`.

* `mirrorClassForInterface` -> MirrorClass

  Returns `InterfaceMirror`.

* `mirrorClassForClass` -> MirrorClass

  Returns `ClassMirror`.

#### Instance Methods

* `reflectee` -> Object

  Returns the object being reflected, aka the reflectee.

* `classMirror` -> [ClassMirror](#classmirror)

  Returns a mirror for the class of the reflectee.

* `behavior` -> [Behavior](#behavior)

  Returns the behavior of the reflectee.

* `layout` -> [Layout](#layout)

  Returns the layout of the reflectee.

---

### ObjectMirror

(a Class)

#### Interfaces

- [Mirror](#mirror)

#### Direct Methods

* `mirrorClassFor`: Object -> MirrorClass

  Returns `ObjectMirror`.
  
  Allows forcing use of `ObjectMirror` instead of the mirror the reflected
  object would request, doing:
  
  ``` foolang
  system reflection of: object in: ObjectMirror
  ```
  
  instead of:
  
  ``` foolang
  system reflection of: object in: Mirror
  ```

---

### ClassMirror

(a Class)

#### Interfaces

- [Mirror](#mirror)

#### Direct Methods

  Returns `ClassMirror`.
  
  Allows forcing use of `ClassMirror` instead of the mirror the reflected object
  would request, doing:
  
  ``` foolang
  system reflection of: object in: ClassMirror
  ```
  
  instead of:
  
  ``` foolang
  system reflection of: object in: Mirror
  ```

#### Instance Methods

* `name` -> Selector

  Returns the name of the class being reflected.

* `instanceBehavior` -> [Behavior](#behavior)

  Returns the behavior of the instances of the class.

* `instanceLayout` -> [Behavior](#behavior)

  Returns the layout of the instances of the class.

---

### InterfaceMirror

(a Class)

#### Interfaces

- [Mirror](#mirror)

#### Direct Methods

* `mirrorClassFor`: Object -> MirrorClass

  Returns `InterfaceMirror`.

  Allows forcing use of `InterfaceMirror` instead of the mirror the reflected object
  would request, doing:
  
  ``` foolang
  system reflection of: object in: InterfaceMirror
  ```
  
  instead of:
  
  ``` foolang
  system reflection of: object in: Mirror
  ```

#### Instance Methods

* `name` -> Selector

  Returns the name of the interface being reflected.

* `instanceBehavior` -> [Behavior](#behavior)

  Returns the behavior of the instances of the class.

---

### Protocol

(an Interface)

Common ancestor of `Interface` and `Class`.

* **required method** `name` -> String

  Returns the name of the procotol.

* **direct method** `includes:` object -> Boolean

  Returns `True` iff _object_ implements the protocol.

---

### Class

(an Interface)

Individual classes are instances of corresponding metaclasses, and
[Metaclass](#metaclass) is an instance of itself. This leads to `Class` being an
[Interface](#interface), which may seem counterintuitive.

Consider:
``` foolang
12 class --> Integer
Integer class --> Integer class (an anonymous metaclass)
Integer class class --> Metaclass
Metaclass class --> Metaclass
```

For `Class includes: anObject` to be true iff _anObject_ is any of the types
of classes above, `Class` must be an interface: they are factually instances
of diverse classes.

#### Interfaces
- [Protocol](#protocol)

#### Dictionary

* **direct method**  \
  `name:` name  \
  `instanceLayout:` layout  \
  `interfaces:` interfaces  \
  `directMethods:` directMethods  \
  `instanceMethods:` instanceMethods  \
  -> [Class](#class)

   Constructs the class _name_, and the metaclass it is an instance of.
   The metaclass holds the direct methods of the class, whereas the class
   holds the instance methods of the class instances.
   
   The constructed class is not defined in the global environment.
   
   The system does not copy down any methods from specified interfaces, but does
   validate that the specified methods fulfill the requirements, including
   inherited ones.
  
   Approximately:
   ``` foolang
   let theMetaclass = Metaclass
                         name: "{name} class"
                         interfaces: [Class]
                         methods: directMethods.
   theMetaclass
       newClassName: name
       layout: instanceLayout
       interfaces: ([Class] append: interfaces) removeDuplicates
       methods: instanceMethods!
   ```

---

### Interface (interface)

is Protocol

* **direct method**  \
  `name:` name  \
  `interfaces:` interfaces  \
  `ownMethods:` ownMethods  \
  `directMethods:` directMethods  \
  `instanceMethods:` instanceMethods  \
  -> [Interface](#interface)

  Constructs a new instance of `Interface`.

  Approximately:
  ``` foolang
  let theMetaclass = Metaclass
                        name: "{name} class"
                        interfaces: [Class]
                        methods: ownMethods
  theMetaclass
      newInterfaceName: name
      interfaces: ([Interface] append: interfaces) removeDuplicates 
      directMethods: directMethods
      instanceMethods: instanceMethods!
  ```

---

### Metaclass (class)

is Class

Metaclasses hold direct methods of classes and interfaces as their instance methods.

* **direct method**  \
  `name:` name  \
  `interfaces:` interfaces  \
  `methods:` methods  \
  -> [Metaclass](#metaclass)

  Constructs a new metaclass, which can be used to construct exactly one
  interface or class.
  
  The _interfaces_ are interfaces that metaclass instances implements directly.
  
  The _methods_ are methods that apply to instances of the metaclass.
  
  The system does not copy down any methods from specified interfaces, but
  does validate that the specified methods fulfill the requirements, including
  inherited ones.
  
* **method**  \
  `newInterfaceName:` name  \
  `interfaces:` interfaces  \
  `directMethods:` directMethods  \
  `instanceMethods:` instanceMethods  \
  -> [Interface](#interface)
  
  Constructs a new interface, which is an instance of the metaclass.
  
  The _interfaces_ are interfaces that interface implements directly.
  
  The _direct methods_ will become metaclass instance methods of
  implementing protocols.
  
  The _instance methods_ will become instance methods of implementing classes.
  
  The system does not copy down any methods from specified interfaces, but
  does validate that the specified methods fulfill the requirements, including
  inherited ones.

* **method**  \
  `newClassName:` name  \
  `layout:` layout  \
  `interfaces:` interfaces  \
  `methods:` methods  \
  -> [Class](#class)
  
  Constructs a new class, which is an instance of the metaclass.

  The _layout_ is the layout for instances of the new class.
  
  The _interfaces_ are interfaces that class implements directly.
  
  The _methods_ are instance methods of the class.
  
  The system does not copy down any methods from specified interfaces, but
  does validate that the specified methods fulfill the requirements, including
  inherited ones.

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

* **method** `classBehavior` -> Procotol

Returns behavior for the class.

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
