use std::path::Path;

use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

use nalgebra::Translation3;

pub fn class_vtable() -> Vtable {
    Vtable::new("SceneNode")
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::new("SceneNode");
    vt.add_primitive_method_or_panic("color:", scene_node_color);
    vt.add_primitive_method_or_panic("texture:from:", scene_node_texture_from);
    vt.add_primitive_method_or_panic("translate:", scene_node_translate);
    vt
}

fn scene_node_color(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        args[0].as_vec(|vec| {
            x = vec[0].send("asFloat", &[], env)?.float() as f32;
            y = vec[1].send("asFloat", &[], env)?.float() as f32;
            z = vec[2].send("asFloat", &[], env)?.float() as f32;
            Ok(())
        })?;
        let mut node = receiver.scene_node().node.borrow_mut();
        node.set_color(x, y, z);
    }
    Ok(receiver.clone())
}

fn scene_node_texture_from(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    kiss3d::resource::TextureManager::get_global_manager(|manager| {
        let path = Path::new(args[1].string_as_str());
        let texture = manager.add(&path, args[0].string_as_str());
        let mut node = receiver.scene_node().node.borrow_mut();
        node.set_texture(texture);
        Ok(())
    })?;
    Ok(receiver.clone())
}

fn scene_node_translate(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    {
        let t = args[0].as_vec(|vec| {
            let x = vec[0].send("asFloat", &[], env)?.float() as f32;
            let y = vec[1].send("asFloat", &[], env)?.float() as f32;
            let z = vec[2].send("asFloat", &[], env)?.float() as f32;
            Ok(Translation3::new(x, y, z))
        })?;
        let mut node = receiver.scene_node().node.borrow_mut();
        node.append_translation(&t);
    }
    Ok(receiver.clone())
}
