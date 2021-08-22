use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use crate::unwind::Unwind;

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("String");
    vt.add_primitive_method_or_panic("toUppercase", string_to_uppercase);
    vt.add_primitive_method_or_panic("append:", string_append_);
    vt.add_primitive_method_or_panic("toString", string_to_string);
    vt.add_primitive_method_or_panic("size", string_size);
    vt.add_primitive_method_or_panic("do:", string_do);
    vt.add_primitive_method_or_panic("codeAt:", string_code_at);
    vt.add_primitive_method_or_panic("from:to:", string_from_to);
    vt.add_primitive_method_or_panic("isEquivalent:", string_is_equivalent);
    vt.add_primitive_method_or_panic("sendTo:with:", string_send_to_with);
    vt
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::for_class("String");
    vt.add_primitive_method_or_panic("new", class_string_new);
    vt.add_primitive_method_or_panic("concat:", class_string_concat);
    vt
}

fn class_string_new(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string(""))
}

fn class_string_concat(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let contents = args[0].as_array("String#from:")?;
    let mut size = 0;
    for s in contents.borrow().iter() {
        size += s.as_str()?.len();
    }
    let mut string = String::with_capacity(size);
    for s in contents.borrow().iter() {
        string.push_str(s.as_str()?);
    }
    Ok(env.foo.into_string(string))
}

fn string_do(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    for ch in receiver.string_as_str().chars() {
        args[0].send("value:", &[env.foo.make_string(&ch.to_string())], env)?;
    }
    Ok(receiver.clone())
}

fn string_send_to_with(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let selector2 = receiver.string_as_str();
    let receiver2 = &args[0];
    let args2 = &args[1].as_array("String#sendTo:with:")?.borrow();
    receiver2.send(selector2, args2, env)
}

fn string_to_uppercase(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let s = receiver.string_as_str().to_string().to_uppercase();
    Ok(env.foo.into_string(s))
}

fn string_append_(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut s = receiver.string_as_str().to_string();
    s.push_str(args[0].string_as_str());
    Ok(env.foo.into_string(s))
}

fn string_to_string(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    Ok(receiver.clone())
}

fn string_size(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_integer(receiver.string_as_str().len() as i64))
}

fn string_from_to(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let data: &str = receiver.string_as_str();
    let from = args[0].integer();
    let i = (from - 1) as usize;
    if from < 1 || data.len() < i {
        return Unwind::error(&format!(
            "String#from:to: -- #from: out of bounds: {}, should be 1-{}",
            from,
            data.len()
        ));
    }
    let to = args[1].integer();
    let j = to as usize;
    if from > to + 1 {
        return Unwind::error(&format!(
            "String#From:to: -- #from: {} is greater than #to+1: {}",
            from, to
        ));
    }
    if data.len() < j {
        return Unwind::error(&format!(
            "String#from:to: -- #to: out of bounds: {}, should be 1-{}",
            to,
            data.len()
        ));
    }
    Ok(env.foo.make_string(&data[i..j]))
}

fn string_code_at(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let data: &str = receiver.string_as_str();
    let arg = args[0].as_i64("String#codeAt: index")?;
    let i = (arg - 1) as usize;
    if arg < 1 || data.len() < arg as usize {
        return Unwind::error(&format!("Index out of bounds for string: {}", arg));
    }
    if data.is_char_boundary(i) {
        if let Some(code) = data[i..].chars().next() {
            Ok(env.foo.make_integer(code as i64))
        } else {
            Unwind::error(&format!("Index out of bounds for string: {}", arg))
        }
    } else {
        Unwind::error(&format!("String#codeAt: {} not at character boundary.", arg))
    }
}

fn string_is_equivalent(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.string_as_str() == args[0].string_as_str()))
}
