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

    flag SupervisorOption { OneForOne, AllForOne, OneForAll, AllForAll }
       -- toString is implicit
       method forAll
          self = OneForAll OR self = AllForAll
    end

seens useful.

The classic C-use cases aren't pointless either:

    enum StandardFD { Stdin=0, Stdout=1, Stderr=2 }
       -- allows: StandardFD value: fd
       -- allows: Stdin value
    end

