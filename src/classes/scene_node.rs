use crate::objects::{Eval, Foolang, Object, Vtable};
use std::path::Path;

use nalgebra::Translation3;

pub fn class_vtable() -> Vtable {
    Vtable::new("class SceneNode")
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("SceneNode");
    vt.def("color:", scene_node_color);
    vt.def("texture:from:", scene_node_texture_from);
    vt.def("translate:", scene_node_translate);
    vt
}

fn scene_node_color(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        args[0].as_vec(|vec| {
            x = vec[0].send("asFloat", &[], foo)?.float() as f32;
            y = vec[1].send("asFloat", &[], foo)?.float() as f32;
            z = vec[2].send("asFloat", &[], foo)?.float() as f32;
            Ok(())
        })?;
        let mut node = receiver.scene_node().node.borrow_mut();
        node.set_color(x, y, z);
    }
    Ok(receiver.clone())
}

fn scene_node_texture_from(receiver: &Object, args: &[Object], _foo: &Foolang) -> Eval {
    kiss3d::resource::TextureManager::get_global_manager(|manager| {
        let path = Path::new(args[1].string_as_str());
        let texture = manager.add(&path, args[0].string_as_str());
        let mut node = receiver.scene_node().node.borrow_mut();
        node.set_texture(texture);
        Ok(())
    })?;
    Ok(receiver.clone())
}

fn scene_node_translate(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    {
        let t = args[0].as_vec(|vec| {
            let x = vec[0].send("asFloat", &[], foo)?.float() as f32;
            let y = vec[1].send("asFloat", &[], foo)?.float() as f32;
            let z = vec[2].send("asFloat", &[], foo)?.float() as f32;
            Ok(Translation3::new(x, y, z))
        })?;
        let mut node = receiver.scene_node().node.borrow_mut();
        node.append_translation(&t);
    }
    Ok(receiver.clone())
}
