# IDE

## Presentation

Ability to look at the code in different ways:

- All methods of this class.
- All methods with this selector (in this package / globally)
- All methods referencing this selector.
- Current method followed the methods on self it invokes.
- Manual selection of methods.
- Methods belonging to this interface.

What should be the default presentation of a class (other than
configurable)?

- Documented methods first?
- Methods grouped by protocol?
- Methods mentioned in class docstring first?

## Source Code

Source organization should be largely automatic. "Go full Java"?

    package/subpackage/ConstantName.foo
    package/subpackage/ClassName/CLASS.foo
    package/subpackage/ClassName/methodName.foo

The benefit of this is that rewriting a method becomes trivial
enough that it cannot really go wrong.

Additionally a kind-of-nice feature of this way of modeling
packages is that manipulation of the hierachy becomes really
trivial...

Then I'm kind of tempted to actually make the method files
just contain:

    @method foo
        ....

so that renaming a class (in absence of type annotations)
requires only renaming the directory.
