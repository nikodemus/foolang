use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use kiss3d;

use std::path::Path;

use nalgebra::{Point3, Vector3};

pub fn class_vtable() -> Vtable {
    Vtable::new("class Window")
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Window");
    vt.def("cube:", window_cube);
    vt.def("close", window_close);
    vt.def("light", window_light);
    vt.def("light:", window_light_arg);
    vt.def("obj:mtl:scale:", window_obj_mtl_scale);
    vt.def("render", window_render);
    vt.def("framerateLimit:", window_framerate_limit);
    vt.def("ifShouldClose:", window_if_should_close);
    vt.def("shouldClose", window_should_close);
    vt
}

fn window_close(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    {
        let mut win = receiver.window().window.borrow_mut();
        win.close();
    }
    Ok(receiver.clone())
}

fn window_cube(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let cube = {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        args[0].as_vec(|vec| {
            x = vec[0].send("asFloat", &[], env)?.float() as f32;
            y = vec[1].send("asFloat", &[], env)?.float() as f32;
            z = vec[2].send("asFloat", &[], env)?.float() as f32;
            Ok(())
        })?;
        let mut win = receiver.window().window.borrow_mut();
        win.add_cube(x, y, z)
    };
    Ok(env.foo.make_scene_node(cube))
}

fn window_obj_mtl_scale(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let obj_path = Path::new(args[0].string_as_str());
    let mtl_path = Path::new(args[1].string_as_str());
    args[2].as_vec(|vec| {
        let x = vec[0].send("asFloat", &[], env)?.float() as f32;
        let y = vec[1].send("asFloat", &[], env)?.float() as f32;
        let z = vec[2].send("asFloat", &[], env)?.float() as f32;
        let mut win = receiver.window().window.borrow_mut();
        let obj = win.add_obj(&obj_path, &mtl_path, Vector3::new(x, y, z));
        Ok(env.foo.make_scene_node(obj))
    })
}

fn window_light(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    {
        let mut win = receiver.window().window.borrow_mut();
        win.set_light(kiss3d::light::Light::StickToCamera);
    }
    Ok(receiver.clone())
}

fn window_light_arg(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    args[0].as_vec(|vec| {
        let x = vec[0].send("asFloat", &[], env)?.float() as f32;
        let y = vec[1].send("asFloat", &[], env)?.float() as f32;
        let z = vec[2].send("asFloat", &[], env)?.float() as f32;
        let mut win = receiver.window().window.borrow_mut();
        win.set_light(kiss3d::light::Light::Absolute(Point3::new(x, y, z)));
        Ok(())
    })?;
    Ok(receiver.clone())
}

fn window_framerate_limit(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let limit = if args[0].is_boolean() {
        None
    } else {
        Some(args[0].send("asInteger", &[], env)?.integer() as u64)
    };
    {
        let mut win = receiver.window().window.borrow_mut();
        win.set_framerate_limit(limit)
    }
    Ok(receiver.clone())
}

fn window_render(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let ok = {
        let mut win = receiver.window().window.borrow_mut();
        win.render()
    };
    Ok(env.foo.make_boolean(ok))
}

fn window_if_should_close(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let should = {
        let win = receiver.window().window.borrow_mut();
        win.should_close()
    };
    if should {
        args[0].send("value", &[], env)
    } else {
        Ok(env.foo.make_boolean(false))
    }
}

fn window_should_close(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let should = {
        let win = receiver.window().window.borrow_mut();
        win.should_close()
    };
    Ok(env.foo.make_boolean(should))
}
