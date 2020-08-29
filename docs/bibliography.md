# Foolang Bibliography 

!> This is a bibliography of interest, NOT "lessons internalized in Foolang". If
there isn't a cursive note explaining a reference's relevance to Foolang, then
it's here more for future reference than anything else.

## Language Design

- 1981 - [Design Principles Behind
  Smalltalk](http://stephane.ducasse.free.fr/FreeBooks/BlueBookHughes/Design%20Principles%20Behind%20Smalltalk.pdf)
  by Daniel Ingalis. _A compelling statement of intent. Foolang doesn't quite follow these Smalltalk
  principles, but being explicit about that would be nice._ 

- 1983 - [Smalltalk-80: The Language and Its Implementation](http://stephane.ducasse.free.fr/FreeBooks/BlueBook/Bluebook.pdf)
  Adale Goldberd and David Robson. 

- 1990 - [A Type System for
  Smalltalk](https://www.researchgate.net/publication/2815088_A_Type_System_for_Smalltalk)
  by Justin O. Graver and Ralph E. Johnson.

- 1993 - [Strongtalk, Typechecking Smalltalk in a Production
  Environment](http://laputan.org/pub/papers/Strongtalk-OOPSLA-93.pdf) by Gilad Bracha and David
  Griswold.

- 1996 - [Subsystems](https://www.instantiations.com/PDFs/OOPSLA96/subsys.pdf)
  by Allen Wirfs-Brock, a proposal from OOPSLA'96 Extending Smalltalk Workshop.
  _Private selectors are the really interesting part._

- 1996 - [A Declarative Model for Defining Smalltalk Programs (OOPSLA'96
  paper)](http://wirfs-brock.com/allen/files/papers/oopsladcl.pdf), by Allen
  Wirfs-Brock, Juanita Ewing, Harold Williams, Brian Wilkerson. See also the
  [OOPSLA'96 presentation slides for the
  paper](http://www.wirfs-brock.com/allen/talks/oopsla96dclslides.pdf), and a
  shorter paper with the same title from [Smalltalk Solutions
  97](https://web.archive.org/web/20200301140324/https://www.instantiations.com/vast/files/archive/Smalltalk-Solutions97/SSDCL1.HTM).
  _These papers describe a model very closely aligned with Foolang's.
  Only major difference is that in Foolang all globals are immutable
  during program execution. Very much worth reading._

- 1997 - [Smalltalk ANSI Standard
  (draft)](https://web.archive.org/web/20200301135851/http://www.math.sfedu.ru/smalltalk/standard/index.html.en)

<a id="scharli2003"></a>
- 2003 - [Traits: Composable Units of
  Behaviour](http://scg.unibe.ch/archive/papers/Scha03aTraits.pdf)
  by Nathanael Schärli, Stéphane Ducasse, Oscar Nierstrasz, and Andrew P. Black.
  _Foolang interfaces are modeled after this paper, and the missing parts are
  probably going to happen in due time._

- 2004 - [Mirrors: Design Principles for Meta-level Facilities of
  Object-Oriented Programming Languages](https://bracha.org/mirrors.pdf) by
  Gilad Bracha and David Ungar. _Foolang does not yet support reflection, but
  when it does it will be use a mirror-based design._

- 2005 - [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf) by
  Roberto Ierusalimschy, Luiz Henrique de Figueiredo, and Waldemar Celes. 
  _While goals of Lua are radically different from Foolang, the careful matching
  of Lua's goals and implementation strategies is noteworthy, as is the design
  of the virtual machine.__

- 2008 - [The Fortress Language Specification](http://www.ccs.neu.edu/home/samth/fortress-spec.pdf) by
  various. _Foolang has quite different design criteria than Fortress, but
  Fortress was an amazing design and well worth learning from._

- 2008 - [The Little Manual of API
  Design](https://people.mpi-inf.mpg.de/~jblanche/api-design.pdf) by Jasmin
  Blanchette. _Library design rather than language design, but close enough.
  Everyone should read this._

- 2009 - [On Understading Data Abstraction,
  Revisited](https://www.cs.utexas.edu/~wcook/Drafts/2009/essay.pdf) by William
  R. Cook. _What is the difference between ADTs and objects? Read this and a
  light will go on. Foolang is very much on the object side of this divide._

- 2017 - [Pony Language](https://www.ponylang.io/)
  _Pony's rejection of ambient authority is where Foolang got the bug. Lots of
  interesting stuff going in Pony! Foolang's more dynamic paradigm rules a lot
  of Pony's coolness out, though. If you're looking for a super interesting
  non-toy language you might want to check out Pony._

- 2017 - [Lunar Programming Language](http://users.rcn.com/david-moon/Lunar/) by
  David Moon.

## Method Dispatch

- 2001 - [Efficient Implementation of Java
  Interfaces](https://yanniss.github.io/M135-18/oopsla01.pdf) by Bowen Alpern,
  Anthony Cocchi, Stephen Fink, David Grove, and Derek Lieber. Describes the
  method Jalapeno JVM uses. (Or at least used in 2001.)

- 2005 - [Efficient Multimethods in a Single Dispatch
  Language](http://www.laputan.org/reflection/Foote-Johnson-Noble-ECOOP-2005.pdf)
  by Brian Foote, Ralph E. Johnson, and James Noble. _An **excellent** discourse
  on multimethods in Smalltalk-like languages. Multimethods following
  assymmetric semantics were in Foolang plans even before I discovered this
  paper. :) From personal CLOS implementation experience I suspect they did
  not give the table approach a fair shake&mdash;it really requires a custom
  hashtable._

- 2007 - [Open, extensible object models](https://www.piumarta.com/software/cola/objmodel2.pdf) by Ian Piumarta
  and Alessandro Warth. _Hugely influential in early genesis of Foolang._

- 2018 - [Interface Dispatch](https://lukasatkinson.de/2018/interface-dispatch/)
  by Lukas Atkinson. Describes interface method calls on C++ (GCC), Java
  (OpenJDK/HotSopt), C# (CLR), and Rust.

## Parsing & Printing

- 1973 - [Top Down Operator Precedence](/papers/pratt.pdf) by Vaughan R. Pratt.
  _Foolang uses a Pratt-style parser._

- 1997 - [A prettier
  printer](https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf)
  by Philip Wadler. _A lovely pretty-printing algorith._

## Resource Management

- 2002 - [Destructors, Finalizers, and Synchronization](https://www.hpl.hp.com/techreports/2002/HPL-2002-335.pdf)
  by Hans-J. Boehm. _This paper is a source of a lot of design-angst to Foolang._

## Unwinding

- 2011 - [Efficient Implementation of Smalltalk Block Returns](http://www.wirfs-brock.com/allen/things/smalltalk-things/efficient-implementation-smalltalk-block-returns)

- 2019 - [Fast and Reliable DWARF Unwindin, and Beyond](https://fzn.fr/projects/frdwarf/)
