### XXX Issues
- how to express type "this must be a protocol inheriting from x"
  (not an instance, but a protocol!)
- how to express type "constant x"

### XXX Compromises (for printing, etc)
- #classOf method in Any (should be `class`, but parsing prevents right now)
- #name method in Class and Interface

### XXX Decisions
- direct methods on classes and interfaces are instance methods on metaclasses
- Class is an interface
- Interface is a interface

### XXX Missing
- Layout and interface modifications via `Mirror#interfaces:`, and `Mirror#layout:`
- MethodDictionaries should link to interfaces in order to maintain
  their signature requirements, and have a "valid" flag somewhere.
- No way to ask "which procotols directly implement this one", or "which
  protocols does this one implement directly" (ie. asking about `is Foo`) since
  that information is lost by the time the metaclasses are constructed.
  One option would be to just pass it in as metadata, and validate that the
  _set_ of interfaces passes in is valid, regardless of their order.

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

### Dependee

(an Interface)

#### Interface Methods

* **required** _onNextChange -> List of: Block

  Returns the list of blocks waiting for next change notification.

* **required** _onEveryChange -> List of: Block

  Returns the list of blocks waiting for every change notification.

* `onEveryChange:` Block

  Arranges _block_ to be evaluated on every change notification.

* `onNextChange:` Block

  Arranges _block_ to be evaluated on next change notification, but
  not subsequently.

* `changeNotification`

  Performs a change notification, evaluating appropriate blocks.

### Reflection

(an Interface)

!> Cannot currently ask "which interfaces does this implement directly",
since that information is not passed through to metaobject constructors!

#### Instance Methods

* `reflectee` -> Object

  Returns the object being reflected, aka reflectee.

* `classReflection` -> [Reflection](#reflection)

  Returns a reflection for the class of the reflectee.

* `layout` -> [Layout](#layout)

  Returns the layout of the reflectee.

* `interfaces` -> Array of: [Interface](#interface)

  Returns the interfaces the reflectee implements.

* `methodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns the method dictionary of the reflectee.

* `implementors` -> Array of: [Interface](#interface)

  Returns an array of procotols that implement the reflectee, false
  if the reflectee is not a protocol.

---

### System Reflection

(a Class)

The built-in reflection class, accessible as `System#reflection`.

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
  on the proxied object instead. Classes implementing `Mirror` typically
  override this to return themselves.

  **NOTE**: Implementations of `mirrorClassUsing:` should not directly refer to
  classes implementing [Mirror](#mirror): that creates unnecessary references to
  `Mirror` in them, preventing useful compiler analysis. Such references can be
  avoided using the `#mirrorClassFor*` messages.

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

* `layout` -> [Layout](#layout)

  Returns the layout of the reflectee.

* `interfaces` -> Array of: [Interface](#interface)

  Returns the interfaces the reflectee implements.

* `methodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns the method dictionary of the reflectee.

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

* `name` -> String

  Returns the name of the reflectee.

* `instanceLayout` -> [Layout](#layout)

  Returns the layout of the instances of the reflectee.

* `instanceInterfaces` -> Array of: [Interface](#interface)

  Returns the interfaces instances of the reflectee implement.

* `instanceMethodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns the method dictionary of instances of the reflectee.

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

* `name` -> String

  Returns the name of the reflectee.

* `instanceInterfaces` -> Array of: [Interface](#interface)

  Returns the interfaces instances of the reflectee implement.

* `instanceMethodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns the method dictionary of instances of the reflectee.

---

### Protocol

(an Interface)

Common ancestor of `Interface` and `Class`.

#### Direct Methods

* `name` -> String

  Returns the name of the procotol.

* `includes:` object -> Boolean

  Returns `True` iff _object_ implements the protocol.

---

### Class

(a Class)

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

Without inheritance of concrete classes, for `Class includes: anObject` to be
true iff _anObject_ is any of the types of classes above, `Class` must be an
interface: they are factually instances of diverse classes.

#### Interfaces

- [Protocol](#protocol)

#### Direct Methods

* `name:` name  \
  `instanceLayout:` layout  \
  `interfaces:` interfaces  \
  `directMethodDictionary:` directMethodDictionary  \
  `instanceMethodDictionary:` instanceMethodDictionary  \
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
                         methodDictionary: directMethodDictionary.
   theMetaclass
       newClassName: name
       layout: instanceLayout
       interfaces: ([Class] append: interfaces) removeDuplicates
       methodDictionary: instanceMethodDictionary!
   ```

---

### Interface

(an Interface)

#### Interfaces

- [Protocol](#protocol)

#### Direct Methods

* `name:` name  \
  `interfaces:` interfaces  \
  `directMethodDictionary:` [MethodDictionary](#methoddictionary)  \
  `instanceMethodDictionary:` [MethodDictionary](#methoddictionary)  \
  -> [Interface](#interface)

  Constructs a new instance of `Interface`.

  Approximately:
  ``` foolang
  let theMetaclass = Metaclass
                        name: "{name} class"
                        interfaces: [Class]
                        methodDictionary: directMethodDictionary
  theMetaclass
      newInterfaceName: name
      interfaces: ([Interface] append: interfaces) removeDuplicates
      methodDictionary: instanceMethodDictionary!
  ```

---

### Metaclass

(a Class)

Metaclasses hold direct methods of classes and interfaces as their instance methods.

#### Interfaces

- [Class](#class)

#### Direct Methods

* `name:` name `methods:` [MethodDictionary](#methoddictionary) -> [Metaclass](#metaclass)

  Constructs a new metaclass, which can be used to construct exactly one
  interface or class.

#### Instance Methods

* `newInterfaceName:` name `interfaces:` interfaces `methods:` methodDictionary -> [Interface](#interface)

  Constructs a new interface, which is an instance of the metaclass.

* `newClassName:` name `layout:` layout `methods:` methodDictionary -> [Class](#class)

  Constructs a new class, which is an instance of the metaclass.

---

### MethodDictionary

(an Interface)

#### Interfaces
- [Dependee](#dependee)

#### Instance Methods

* `at:` Selector `ifNone:` Block -> [Method](#method)

  Returns the method associated with the _selector_, evaluating
  the _block_ if no method is yet associated with the selector.

* `put:` Method `at:` Selector -> [Method](#method)

  Associated the method with the selector.

  Notifies of change on the method dictionary.

  If there was a method already associated with the selector, also notifies of
  change on the method.

* `selectors` -> Array of: Selector

  Returns selectors with methods associated with them.

---

### Layout

(an Interface)

#### Instance Methods

* `host` -> Class

Returns the class whose layout this is.

* `slots` -> Array of: [Slot](#interface-slot)

Returns and array of slots in the same order as they appeared in
the class definition.

* `allocate:` initialValues -> Any

Returns an instance of `#host` class with given initial values corresponding to
slots 1:1.

---

### Slot

(an Interface)

#### Instance Methods

* `name` -> Selector

Returns the name of this slot.

* `type` -> Type

Returns the type constraint of this slot.

* `read:` instance -> Any

Returns the value of this slot in _instance_. The returned value is of `#type`.

* `write:` value `to:` instance -> Any

Writes _value_ to this slot in _instance_. Returns _value_.

---

### Method

(an Interface)

#### Instance Methods

* `signature` -> [Signature]

Returns the signature of the method.

* `invokeOn:` receiver `withArguments:` arguments -> Any

Invokes the method using _receiver_ and _arguments_. Returns a value
consistent with `signature returnType`.

---

### Signature

(a Class)

#### Direct Methods

* `argumentTypes:` argumentTypes `returnType:` returnType ->
  [Signature](#signature)

  Returns a new signature.

#### Instance Methods

* `argumentTypes` -> Array of: Type

  Returns argument types required by the signature.

* `returnType` -> Type

  Returns the return type promised by the signature.

* `implements:` otherSignature

  Returns true iff the argument types of the signatures match exactly,
  and that the return type of the receiver is a subtype of the return
  type of the other signature.

---

## Summary

- _"How are methods resolved?"_ Probably using C3 linearization, but class
  constructor is responsible - as long as they're consistent with all inherited
  interfaces.
- _"Is there a `super` or equivalent?"_ Not answered yet.
- _"How to build classes and interfaces programmatically?"_ By using direct
  methods on `Class` and `Interface`, or using `Metaclass` directly.
- _"What are the metaobjects that describe these issues?"_ See above.
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
