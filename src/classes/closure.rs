use crate::eval::{Binding, Env};
use crate::expr::*;
use crate::objects::{Arg, Eval, Object, Signature, Source, Vtable};
use crate::unwind::Unwind;
use std::collections::HashMap;

use std::cmp::Eq;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Closure {
    pub name: String,
    pub env: Env,
    pub params: Vec<Arg>,
    pub body: Expr,
    pub signature: Signature,
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Closure {}

impl Hash for Closure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl Closure {
    pub fn extend_env(&self, name: &str, value: &Object) -> Rc<Closure> {
        Rc::new(Closure {
            name: self.name.clone(),
            env: self.env.bind(name, Binding::untyped(value.clone())),
            params: self.params.clone(),
            body: self.body.clone(),
            signature: self.signature.clone(),
        })
    }

    pub fn apply(&self, receiver: Option<&Object>, args: &[Object]) -> Eval {
        let mut symbols = HashMap::new();
        if self.params.len() != args.len() {
            return Unwind::error_at(
                // FIXME: call-site would be 1000 x better...
                self.body.source_location(),
                &format!(
                    "Argument count mismatch, {} wanted {}, got {}: {:?}",
                    &self.name,
                    self.params.len(),
                    args.len(),
                    args,
                ),
            );
        }
        for ((arg, vt), obj) in self
            .params
            .iter()
            .zip(&self.signature.parameter_types)
            .zip(args.into_iter().map(|x| (*x).clone()))
        {
            let binding = match vt {
                None => Binding::untyped(obj),
                Some(ref typed) => Binding::typed(typed.clone(), obj, &self.env)?,
            };
            symbols.insert(arg.name.clone(), binding);
        }
        let env = self.env.extend(symbols, receiver);
        let ret = env.eval(&self.body);
        // println!("apply return: {:?}", &ret);
        let result = match ret {
            Ok(value) => value,
            Err(Unwind::ReturnFrom(ref ret_env, ref value)) if ret_env == &env.env_ref => {
                value.clone()
            }
            Err(unwind) => {
                return Err(unwind);
            }
        };
        if let Some(typed) = &self.signature.return_type {
            typed.send("typecheck:", &[result], &self.env).source(&self.body.source_location())
        } else {
            Ok(result)
        }
    }
}

pub fn vtable() -> Vtable {
    let vt = Vtable::for_instance("Closure");
    // FUNDAMENTAL
    vt.add_primitive_method_or_panic("apply:", closure_apply_array);
    vt.add_primitive_method_or_panic("signature", closure_signature);
    vt.add_primitive_method_or_panic("finally:", closure_finally);
    vt.add_primitive_method_or_panic("arity", closure_arity);
    vt.add_primitive_method_or_panic("onPanic:", closure_on_panic);
    vt.add_primitive_method_or_panic("loop", closure_loop);
    vt
}

// FUNDAMENTAL METHODS

fn closure_apply_array(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let array = args[0].as_array("Closure#apply:")?.borrow();
    receiver.closure_ref().apply(None, &array)
}

fn closure_signature(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    // FIXME: return value too?
    let mut sigtypes = Vec::new();
    let any = env.find_type("Any")?;
    for t in &receiver.closure_ref().signature.parameter_types {
        match t {
            Some(o) => sigtypes.push(o.clone()),
            None => sigtypes.push(any.clone()),
        };
    }
    Ok(env.foo.into_array(sigtypes, None))
}

fn closure_arity(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_integer(receiver.closure_ref().params.len() as i64))
}

fn closure_finally(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let res = receiver.closure_ref().apply(None, &[]);
    args[0].closure_ref().apply(None, &[])?;
    res
}

fn closure_on_panic(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.closure_ref().apply(None, &[]);
    if let Err(Unwind::Panic(error, loc)) = res {
        let panic_class = match env.get("Panic") {
            None => panic!("Panic class not defined!"),
            Some(obj) => obj,
        };
        let panic_obj = panic_class.send(
            "description:context:",
            &[env.foo.into_string(error.what()), env.foo.into_string(loc.context())],
            env,
        )?;
        args[0].send("value:", &[panic_obj], env)
    } else {
        res
    }
}

fn closure_loop(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    loop {
        receiver.closure_ref().apply(None, &[])?;
    }
}
