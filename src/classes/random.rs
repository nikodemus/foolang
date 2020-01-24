use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable};
use crate::unwind::Unwind;

use rand::prelude::*;

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub struct Random {
    rng: RefCell<StdRng>,
}

impl Random {
    pub fn borrow(&self) -> Ref<StdRng> {
        self.rng.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<StdRng> {
        self.rng.borrow_mut()
    }
}

impl PartialEq for Random {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Random");
    vt.def("integer", random_integer);
    vt.def("float", random_float);
    vt.def("boolean", random_boolean);
    vt
}

pub fn class_vtable() -> Vtable {
    let mut vt = Vtable::new("class Random");
    vt.def("new", class_random_new);
    vt.def("new:", class_random_new_arg);
    vt
}

pub fn as_random<'a>(obj: &'a Object, ctx: &str) -> Result<&'a Random, Unwind> {
    match &obj.datum {
        Datum::Random(ref random) => Ok(random),
        _ => Unwind::error(&format!("{:?} is not a Random ({})", obj, ctx)),
    }
}

fn class_random_new(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(Object {
        vtable: Rc::clone(&env.foo.random_vtable),
        datum: Datum::Random(Rc::new(Random {
            rng: RefCell::new(StdRng::from_entropy()),
        })),
    })
}

fn class_random_new_arg(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let arg = args[0].as_u64("seed in Random##new:")?;
    Ok(Object {
        vtable: Rc::clone(&env.foo.random_vtable),
        datum: Datum::Random(Rc::new(Random {
            rng: RefCell::new(StdRng::seed_from_u64(arg)),
        })),
    })
}

fn random_integer(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let i: i64 = receiver.as_random("receiver in Random#integer")?.borrow_mut().gen();
    Ok(env.foo.make_integer(i))
}

fn random_float(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let f: f64 = receiver.as_random("receiver in Random#float")?.borrow_mut().gen();
    Ok(env.foo.make_float(f))
}

fn random_boolean(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let b: bool = receiver.as_random("receiver in Random#float")?.borrow_mut().gen();
    Ok(env.foo.make_boolean(b))
}
