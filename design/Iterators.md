External or internal?

Classic ST style is internal.

Early returns and such are a problem, though.

array do: { ... }

==> array iter do: { ... }

==> let iter := array iter
    Do loop: {
      let next = iter next
      next ifNothing: { return }
      block : next
    }

==> let res = Array new: array size
    let index = 0
    array do: { |elt|
      res[index] = elt
      index += 1
    }
    res

map: let iter := array iter
     let res = Array new: array size
     {
       let next := iter next
       next ifNothing: {
         return res
       }
     }
