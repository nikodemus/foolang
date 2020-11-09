use crate::eval::{Binding, Env, EnvRef, SymbolTable};
use crate::expr::*;
use crate::objects::{Arg, Eval, Object, Signature, Source, Vtable};
use crate::unwind::Unwind;
use std::collections::HashMap;

use std::cmp::Eq;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Closure {
    pub name: String,
    pub env_ref: EnvRef,
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
    pub fn apply(&self, receiver: Option<&Object>, args: &[Object], send_env: &Env) -> Eval {
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
        if args.len() == 0 {
            return self.apply0(receiver, send_env);
        }
        if args.len() == 1 {
            return self.apply1(receiver, args, send_env);
        }
        let closure_env = Env {
            env_ref: self
                .env_ref
                .extend(SymbolTable::Big(HashMap::with_capacity(args.len())), receiver),
            foo: send_env.foo.clone(),
        };
        for ((arg, vt), obj) in self
            .params
            .iter()
            .zip(&self.signature.parameter_types)
            .zip(args.into_iter().map(|x| (*x).clone()))
        {
            let binding = match vt {
                None => Binding::untyped(obj),
                Some(ref typed) => Binding::typed(typed.clone(), obj, &closure_env)?,
            };
            closure_env.ensure_binding(&arg.name, binding);
        }
        let ret = closure_env.eval(&self.body);
        let result = match ret {
            Ok(value) => value,
            Err(Unwind::ReturnFrom(ref ret_env, ref value)) if ret_env == &closure_env.env_ref => {
                value.clone()
            }
            Err(unwind) => {
                return Err(unwind);
            }
        };
        if let Some(typed) = &self.signature.return_type {
            typed.send("typecheck:", &[result], &closure_env).source(&self.body.source_location())
        } else {
            Ok(result)
        }
    }

    pub fn apply0(&self, receiver: Option<&Object>, send_env: &Env) -> Eval {
        let closure_env = Env {
            env_ref: self.env_ref.extend(SymbolTable::Empty, receiver),
            foo: send_env.foo.clone(),
        };
        let ret = closure_env.eval(&self.body);
        let result = match ret {
            Ok(value) => value,
            Err(Unwind::ReturnFrom(ref ret_env, ref value)) if ret_env == &closure_env.env_ref => {
                value.clone()
            }
            Err(unwind) => {
                return Err(unwind);
            }
        };
        if let Some(typed) = &self.signature.return_type {
            typed.send("typecheck:", &[result], &closure_env).source(&self.body.source_location())
        } else {
            Ok(result)
        }
    }

    pub fn apply1(&self, receiver: Option<&Object>, args: &[Object], send_env: &Env) -> Eval {
        let closure_env = Env {
            env_ref: self.env_ref.extend(SymbolTable::Empty, receiver),
            foo: send_env.foo.clone(),
        };
        let obj = args[0].clone();
        let binding = match &self.signature.parameter_types[0] {
            None => Binding::untyped(obj),
            Some(ref typed) => Binding::typed(typed.clone(), obj, &closure_env)?,
        };
        closure_env.ensure_binding(&self.params[0].name, binding);
        let ret = closure_env.eval(&self.body);
        let result = match ret {
            Ok(value) => value,
            Err(Unwind::ReturnFrom(ref ret_env, ref value)) if ret_env == &closure_env.env_ref => {
                value.clone()
            }
            Err(unwind) => {
                return Err(unwind);
            }
        };
        if let Some(typed) = &self.signature.return_type {
            typed.send("typecheck:", &[result], &closure_env).source(&self.body.source_location())
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

fn closure_apply_array(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let array = args[0].as_array("Closure#apply:")?.borrow();
    receiver.closure_ref().apply(None, &array, env)
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

fn closure_finally(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.closure_ref().apply(None, &[], env);
    args[0].closure_ref().apply(None, &[], env)?;
    res
}

fn closure_on_panic(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.closure_ref().apply(None, &[], env);
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

fn closure_loop(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    loop {
        receiver.closure_ref().apply(None, &[], env)?;
    }
}
