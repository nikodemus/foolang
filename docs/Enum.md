# Foolang Enums

Erlang supervisors OneForOne constant made me think.

Constants are better as flags than literals Because
then the compiler can do checking.

However:

    class SupervisorOption { name }
       method toString
          name
    end

    define OneForOne = SupervisorOption name: "OneForOne"

gets old really fast.

So something like:

    enum SupervisorOption { OneForOne, AllForOne, OneForAll, AllForAll }
       -- toString is implicit
       method isForAll
          { self is OneForAll } or: { self is AllForAll }
    end

seens useful, plus:

    extend OneForOne
       method kill
          ...
    end

The classic C-use cases aren't pointless either:

    enum StandardFD { Stdin=0, Stdout=1, Stderr=2 }
       -- allows: StandardFD value: fd
       -- allows: Stdin value
    end

Then again, this seems all like a convenience, not actually required.
