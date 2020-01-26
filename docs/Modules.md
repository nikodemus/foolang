# Foolang Modules

Modules are units of import: files and directories of Foolang code.

Namespaces allow same global name to refer to different bindings in
different contexts.

There is often a 1:1 relationship between modules and namespaces, and
as such we tend to use the term module a bit loosely for both.

## Overview

Eg. assuming files

    ~/foo/my/utils/text.foo:
        define Grep = ...
        define Tail = ...

    ~/foo/my/utils/math.foo:
        define Sin = ...
        define Cos = ...

and commandline:

    foo --module=~/foo/utils

the import statement

    import utils

loads the module utils, giving access to both submodules text and math,
which in turn contain the toplevel objects, accessible as:

    utils.text.Grep
    utils.text.Tail
    utils.math.Sin
    utils.math.Cos

_Modules are not first-class objects_: the following names
are do are unbound:

    utils
    utils.text
    utils.math

To inspect contents of a module use reflection:

    let utils = foolang.reflection.ModuleMirror reflect: "utils".
    utils each: { stdout println: _ }

==>

    #<Module utils.text: Grep, Tail>
    #<Module utils.math: Sin, Cos>

etc.

## Syntax of Import Statements

Currently imports are restricted to toplevel.

Import statements use syntax

    import module[.selector] [as: alias]

Selector can be

1. a sub-module like `text` or `math` in Overview above, or global
   name in the module or sub-module.
2. '*' denoting all sub-modules or global names in the module or
   sub-module.

Aliases are not allowed with * imports. Alias must be a legal identifier for the import:

- When importing a module it must be a lower-case identifier.
- When importing a global name it must be an upper-case identifier.

Imports which create conflicts are not allowed.

Examples:

    import utils.text --> text.Grep, text.Tail

    import utils.text.Grep --> Grep

    import utils.* --> text.Grep, text.Tail, math.Sin, math.Cos

    import utils.text.* --> Grep, Tail

    import utils.text as: txt --> txt.Grep, txt.Tail

    import utils.text.Grep as: TextGrep --> TextGrep

    import utils.text.*
    import other.Grep shadowing: utils.text.Grep

Imported names are not transitively visible. To make an imported name
visible to modules importing this one, add a definition:

    import utils

    define Grep = utils.text.Grep

## Future Work

- Shadowing
- Importing a list of names
- Look at Dylan and Scheme and Fortress and think hard.
- Packages and distributions

## References

- [Namespaces in GNU Smalltalk](https://www.gnu.org/software/smalltalk/manual/html_node/Namespaces.html)




