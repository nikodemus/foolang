# Foolang Bibliography 

!> This is a bibliography of interest, NOT "lessons internalized in Foolang". If
there isn't a cursive note explaining a reference's relevance to Foolang, then
it's here more for future reference than anything else.

## Language Design

- 1981 - [Design Principles Behind
  Smalltalk](http://stephane.ducasse.free.fr/FreeBooks/BlueBookHughes/Design%20Principles%20Behind%20Smalltalk.pdf)

- 1983 - [Smalltalk-80: The Language and Its Implementation](http://stephane.ducasse.free.fr/FreeBooks/BlueBook/Bluebook.pdf)
  Adale Goldberd and David Robson

- 1989 - [Reflective Facilities in
  Smalltalk-80](http://www.laputan.org/ref89/ref89.html) by Brian Foote and
  Ralph E. Johnson.

- 1989 - [An Environment for Literate Smalltalk
  Programming](https://heim.ifi.uio.no/~trygver/1989/1989.10.1-LiterateProgrammingOOPSLA.pdf)
  
- 1993 - [Strongtalk, Typechecking Smalltalk in a Production
  Environment](http://laputan.org/pub/papers/Strongtalk-OOPSLA-93.pdf) by Gilad Bracha and David
  Griswold.

- 1993 - [The Early History of
  Smalltalk](http://worrydream.com/EarlyHistoryOfSmalltalk/) by Alan Kay.

- 1996 - [A Declarative Model for Defining Smalltalk Programs (OOPSLA'96
  paper)](http://wirfs-brock.com/allen/files/papers/oopsladcl.pdf), by Allen
  Wirfs-Brock, Juanita Ewing, Harold Williams, Brian Wilkerson. See also the
  [OOPSLA'96 presentation slides for the
  paper](http://www.wirfs-brock.com/allen/talks/oopsla96dclslides.pdf), and a
  shorter paper with the same title from [Smalltalk Solutions
  97](https://web.archive.org/web/20200301140324/https://www.instantiations.com/vast/files/archive/Smalltalk-Solutions97/SSDCL1.HTM).

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
