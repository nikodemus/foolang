### XXX Missing
- Interface modifications via `Mirror#interfaces:`
- "Which procotols directly implement this one" is not availble.
- "Which protocols does this directly implement" is not available.

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
selector, but different implementations, which one gets used?)

How to build classes programmatically?

How to avoid breaking encapsulation so that authority of the system object isn't
subverted by stealing authorizing objects through introspection or intercession?

How can the self-hosted interpreter access slots of the classes it creates?

Are there metaobjects that describe these issues, and if so, what are they?

## Proposal

A simple metaobject protocol, without providing much in the way of readymade
building blocks:

- Direct methods on classes and interfaces are instance methods on
  metaclasses.
- Access to metalevel objects that allow reflection and intercession
  is controlled through `Mirror` and `Reflection` interfaces, putting
  the authority under system object's control and allowing compiler
  visibility into reflection.
- Main metaobjects are:
  - Layouts, granting access to data.
  - Method dictionaries, granting access to behavior.

---

### Any

(an Interface)

#### Direct Methods

* `includes:` object -> Boolean

  Returns True.

#### Instance Methods

* `classOf` -> [Class](#class)

  Returns the class of the object. (Name will change to `class` once syntax
  allows.)

### ChangeSource

(an Interface)

#### Instance Methods

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

* `layout` -> [Layout](#layout)

  Returns a layout object for the reflectee.

  Depending on the specific mirror class this may be read-only. Of the built-in
  mirror classes only `ObjectMirror` returns a read-only layout.

  Depending on the specific mirror class the slots in the layout may be
  restricted to specific instances. Of the built-in mirror classes only
  `ObjectMirror` returns a restricted layout like that.

* `interfaces` -> Array of: [Interface](#interface)

  Returns the interfaces the reflectee implements.

* `methodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns a method dictionary object for the reflectee.

  Depending on the specific mirror class this may be read-only. Of the built-in
  mirror classes only `ObjectMirror` returns a read-only method dictionary.

  Depending on the specific mirror class the methods accessed through the method
  dictionary may be restricted to specific instances. Of the built-in mirror
  classes only `ObjectMirror` returns a restricted method dictionary like that.

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

#### Instance Methods

* `layout` -> [Layout](#layout)

  Returns a read-only layout for the reflectee, whose slots are restricted
  for use with the reflectee.

  **Rationale**: Being granted a mirror on an instance should not grant the
  ability to access arbitrary slots of other instances of the same class.


* `methodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns a read-only method dictionary for the reflectee, restricted to the
  reflectee.

  **Rationale**: Being granted a mirror on an instance should not grant the
  ability to change behavior of other instances of the same class, or invoke
  internal methods on them.

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

* `instanceLayout` -> [Layout](#layout)

  Returns a layout for the instances of the reflectee.

* `instanceInterfaces` -> Array of: [Interface](#interface)

  Returns the interfaces instances of the reflectee implement.

* `methodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns a read-write method dictionary for the reflectee.

* `instanceMethodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns a read-write method dictionary for instances of the reflectee.

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

* `instanceInterfaces` -> Array of: [Interface](#interface)

  Returns the interfaces instances of the reflectee implement.

* `methodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns a read-write method dictionary for the reflectee.

* `instanceMethodDictionary` -> [MethodDictionary](#methoddictionary)

  Returns a read-write method dictionary for instances of the reflectee.

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

   The method dictionaries may not have any requirements associated with them at
   this time: the system will add in the requirements from inherited interfaces.
   Requirements specific to the new class must be added afterwards.

   Approximately:
   ``` foolang
   let theClassClass = Metaclass
                         name: "{name} class"
                         interfaces: [Class]
                         methodDictionary: directMethodDictionary.
   theClassClass
       newClassName: name
       layout: instanceLayout
       interfaces: ([Class] append: interfaces)
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
  `ownMethodDictionary:` [MethodDictionary](#methoddictionary)  \
  `directMethodDictionary:` [MethodDictionary](#methoddictionary)  \
  `instanceMethodDictionary:` [MethodDictionary](#methoddictionary)  \
  -> [Interface](#interface)

  Constructs a new `Interface` and its associated metaclasses and
  metainterface.

  The method dictionaries may not have any requirements associated with them at
  this time: the system will add in the requirements from inherited interfaces.
  Requirements specific to the new interface must be added afterwards.

  Approximately:
  ``` foolang
  let metaInterfaceClass = Metaclass
                               name: "{name} interface class"
                               interfaces: [Class]
                               methodDictionary: InterfaceClassDictionary.
  let metaInterface = metaInterfaceClass
                          name: "{name} interface"
                          interfaces: [Interface]
                          methodDictionary: directMethodDictionary.
  let interfaceMetaclass = Metaclass
                              name: "{name} class"
                              interfaces: [Class]
                              methodDictionary: ownMethodDictionary.
  interfaceMetaclass
      newInterfaceName: name
      interfaces: ([theInterfaceInterFace] append: interfaces)
      methodDictionary: instanceMethodDictionary!
  ```

#### Instance Methods

* `interfaceOf`

  Returns the interface object describing the receiver itself. (This the own
  direct methods of the interface are described by this object.)

---

### Metaclass

(a Class)

Metaclasses hold direct methods of classes and interfaces as their instance methods.

#### Interfaces

- [Class](#class)

#### Direct Methods

* `name:` name `interfaces:` interfaces `methodDictionary:` [MethodDictionary](#methoddictionary) -> [Metaclass](#metaclass)

  Constructs a new metaclass, which can be used to construct exactly one
  interface or class.

  The method dictionary may not have any requirements associated with it
  this time: the system will add in the requirements from inherited interfaces.
  Requirements specific to the metaclass must be added afterwards.

#### Instance Methods

* `newInterfaceName:` name `interfaces:` interfaces `methods:` methodDictionary -> [Interface](#interface)

  Constructs a new interface, which is an instance of the metaclass.

  The method dictionary may not have any requirements associated with it
  this time: the system will add in the requirements from inherited interfaces.
  Requirements specific to the interface must be added afterwards.

* `newClassName:` name `layout:` layout `methods:` methodDictionary -> [Class](#class)

  Constructs a new class, which is an instance of the metaclass.

  The method dictionary may not have any requirements associated with it
  this time: the system will add in the requirements from inherited interfaces.
  Requirements specific to the class must be added afterwards.

---

### MethodDictionary

(an Interface)

```
-- FAILS: instance grants only read-access to the instance method
-- dictionary!
(system reflection of: 42 in: Mirror)
  methodDictionary put: MyMethod at: #boop

-- FAILS: Bad adder signature does not match required signature.
(system reflection of: Integer in: Mirror)
  instanceMethodDictionary put: BadAdder at: #+

(system reflection of: Number in: Mirror)
```

#### Interfaces

- [ChangeSource](#ChangeSource)

#### Direct Methods

* `new`

  Creates a new empty method dictionary.

#### Instance Methods

* `readOnly`

  Returns a read-only wrapper for the receiver, unless it is already read-only,
  in which case it returns the receiver.

* `isReadonly`

  Returns True iff the receiver is read-only.

* `restrictTo:` object

  Returns a restriced method dictionary which wraps any methods read from
  it so that they can only be invoked on the _object_.

* `isRestricted`

  Returns True iff the receiver is restricted to a specific object.

* `restrictedTo`

  Returns the object to which the receiver is restricted, or raises an
  error if the the receiver is not restricted.

* `isComplete` -> Boolean

  Returns true iff there is a method for each requirement in the dictionary.

* `requirementAt:` Selector `ifNone:` Block -> [Requirement](#requirement)

  Returns a requirement object describing requirements for method associated
  with the given selector, or evaluates the block if the selector has no
  associated requirements.

* `putRequirement:` [Requirement](#requirement) `at:` Selector ->
  [Requirement](#requirement)

  Raises an error if there is a pre-existing requirement whose owning method
  dictionary is not the receiver, of if current method associated with the
  receiver does not fulfill the requirement.

  Otherwise sets the requirement for the given selector.

  Notifies of change on the method dictionary.

  If there was a requirement already associated with the selector, also
  notifies of change on the old requirement.

* `removeRequirementAt:` Selector

  Raises an error if there is a pre-existing requirement whose owning method
  dictionary is not the receiver.

  Otherwise removes the requirement for the given selector, if any.

  Notifies of change on the method dictionary.

  If there was a requirement already assocated with the selector, also
  notifies of change on the old requirement.

* `methodAt:` Selector `ifNone:` Block -> [Method](#method)

  Returns the method associated with the _selector_, evaluating the _block_ if
  no method is yet associated with the selector.

* `putMethod:` Method `at:` Selector -> [Method](#method)

  Verifies that the method complies with current requirements on the selector,
  then associates the method with the selector.

  Notifies of change on the method dictionary.

  If there was a method already associated with the selector, also notifies of
  change on the method.

* `removeMethod:` Selector

  Raises a warning if there is a requirement associated with the method.

  Removes the method associated with the _selector_, if any.

  Notifies of change on the method dictionary.

  If there was a method already associated with the selector, also notifies of
  change on the method.

* `do:` Block

  Evaluates the block with each selector and requirement and method associated
  with it. (If either requirement or method is not set, False is used instead.)

---

### Layout

(an Interface)

#### Interfaces

- [ChangeSource](#changesource)

#### Instance Methods

* `readOnly`

  Returns a read-only wrapper for the receiver, unless it is already read-only,
  in which case it returns the receiver.

* `isReadonly`

  Returns True iff the receiver is read-only.

* `restrictTo:` object

  Returns a restriced layout which wraps slots so it so that they can only be
  used to access the _object_.

* `isRestricted`

  Returns True iff the receiver is restricted to a specific object.

* `restrictedTo`

  Returns the object to which the receiver is restricted, or raises an
  error if the the receiver is not restricted.

* `slots` -> Array of: [Slot](#interface-slot)

  Returns and array of slots in the same order as they appeared in
  the class definition.

* `slots:` Array of: [Slot](#interface-slot)

  Sets slots of the layout. Causes any accesses to previously allocated
  instances with the old layout to trap, remapping to the new layout.

  Slots which did not previously exist _must_ have a default value
  block, used to compute initial value for pre-existing instances.

  Notifies of change on layout.

  If pre-existing slots are removed or change positions, notifies of
  change on those slots.

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
