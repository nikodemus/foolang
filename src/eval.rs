use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::objects::{
    read_instance_variable, write_instance_variable, Arg, Closure, Eval, Foolang, Object, Source,
    Vtable,
};
use crate::parse::{
    Array, Assign, ClassDefinition, Expr, Global, Import, Literal, Message, Parser, Return, Var,
};
use crate::tokenstream::Span;
use crate::unwind::Unwind;

#[derive(Debug)]
pub struct MethodFrame {
    pub names: RefCell<HashMap<String, Binding>>,
    pub receiver: Object,
}

#[derive(Debug)]
pub struct BlockFrame {
    pub names: RefCell<HashMap<String, Binding>>,
    // Innermost lexically enclosing frame
    pub parent: Option<Frame>,
    // Lexically enclosing method frame
    pub home: Option<Frame>,
}

impl PartialEq for MethodFrame {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl PartialEq for BlockFrame {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

// FIXME:
//  Frame {
//    names:
//    context: BlockContext | MethodContext
//  }
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    MethodFrame(Rc<MethodFrame>),
    BlockFrame(Rc<BlockFrame>),
}

impl Frame {
    fn new(
        names: HashMap<String, Binding>,
        parent: Option<Frame>,
        receiver: Option<Object>,
    ) -> Frame {
        match receiver {
            None => {
                let home = match &parent {
                    None => None,
                    Some(p) => p.home(),
                };
                Frame::BlockFrame(Rc::new(BlockFrame {
                    names: RefCell::new(names),
                    parent,
                    home,
                }))
            }
            Some(receiver) => {
                assert!(parent.is_none());
                Frame::MethodFrame(Rc::new(MethodFrame {
                    names: RefCell::new(names),
                    receiver,
                }))
            }
        }
    }

    fn names(&self) -> &RefCell<HashMap<String, Binding>> {
        match self {
            Frame::MethodFrame(method_frame) => &method_frame.names,
            Frame::BlockFrame(block_frame) => &block_frame.names,
        }
    }

    fn home(&self) -> Option<Frame> {
        match self {
            Frame::MethodFrame(_) => Some(self.clone()),
            Frame::BlockFrame(block_frame) => block_frame.home.clone(),
        }
    }

    fn receiver(&self) -> Option<&Object> {
        match self {
            Frame::MethodFrame(method_frame) => Some(&method_frame.receiver),
            Frame::BlockFrame(block_frame) => {
                match &block_frame.home {
                    // FIXME: None as span
                    None => None,
                    Some(frame) => frame.receiver(),
                }
            }
        }
    }

    fn parent(&self) -> Option<Frame> {
        match self {
            Frame::MethodFrame(_) => None,
            Frame::BlockFrame(block_frame) => block_frame.parent.clone(),
        }
    }

    fn set(&self, name: &str, value: Object) -> Option<Eval> {
        match self.names().borrow_mut().get_mut(name) {
            Some(binding) => Some(binding.assign(value)),
            None => match self.parent() {
                Some(parent) => parent.set(name, value),
                None => None,
            },
        }
    }

    fn get(&self, name: &str) -> Option<Object> {
        match self.names().borrow().get(name) {
            Some(binding) => return Some(binding.value.clone()),
            None => match self.parent() {
                Some(parent) => parent.get(name),
                None => None,
            },
        }
    }
}

pub struct Env<'a> {
    foo: &'a Foolang,
    frame: Frame,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    vtable: Option<Rc<Vtable>>,
    pub value: Object,
}

impl Binding {
    pub fn untyped(init: Object) -> Binding {
        Binding {
            vtable: None,
            value: init,
        }
    }
    fn typed(vtable: Rc<Vtable>, init: Object) -> Binding {
        Binding {
            vtable: Some(vtable),
            value: init,
        }
    }
    fn assign(&mut self, value: Object) -> Eval {
        if let Some(vtable) = &self.vtable {
            if &value.vtable != vtable {
                return Unwind::type_error(value, vtable.name.clone());
            }
        }
        self.value = value.clone();
        Ok(value)
    }
}

impl<'a> Env<'a> {
    pub fn new(foo: &Foolang) -> Env {
        Env::from_parts(foo, HashMap::new(), None, None)
    }

    pub fn eval(&self, expr: &Expr) -> Eval {
        use Expr::*;
        match expr {
            Array(array) => self.eval_array(array),
            Assign(assign) => self.eval_assign(assign),
            Bind(name, typename, value, body) => self.eval_bind(name, typename, value, body),
            Block(_, params, body, rtype) => self.eval_block(params, body, rtype),
            Cascade(receiver, chains) => self.eval_cascade(receiver, chains),
            ClassDefinition(definition) => self.eval_class_definition(definition),
            Const(_, literal) => self.eval_literal(literal),
            Eq(_, left, right) => self.eval_eq(left, right),
            Global(global) => self.eval_global(global),
            Import(import) => self.eval_import(import),
            Return(ret) => self.eval_return(ret),
            Chain(receiver, messages) => self.eval_chain(receiver, messages),
            Seq(exprs) => self.eval_seq(exprs),
            Typecheck(_, expr, typename) => self.eval_typecheck(expr, typename),
            Var(var) => self.eval_var(var),
        }
    }

    fn from_parts(
        foo: &'a Foolang,
        names: HashMap<String, Binding>,
        parent: Option<Frame>,
        receiver: Option<Object>,
    ) -> Env<'a> {
        Env {
            foo,
            frame: Frame::new(names, parent, receiver),
        }
    }

    fn eval_array(&self, array: &Array) -> Eval {
        let mut data = Vec::new();
        for elt in &array.data {
            data.push(self.eval(elt)?);
        }
        Ok(self.foo.into_array(data))
    }

    fn eval_bind(
        &self,
        name: &String,
        typename: &Option<String>,
        expr: &Expr,
        body: &Option<Box<Expr>>,
    ) -> Eval {
        let binding = match typename {
            None => Binding::untyped(self.eval(expr)?),
            Some(typename) => {
                let class = self.find_class(typename, expr.span())?.class();
                Binding::typed(class.instance_vtable.clone(), self.eval_typecheck(expr, typename)?)
            }
        };
        let value = match body {
            None => Err(binding.value.clone()),
            Some(expr) => Ok(expr),
        };
        match self.foo.workspace {
            Some(ref workspace) if self.frame.receiver().is_none() => {
                // Toplevel let in workspace
                {
                    let mut table = workspace.borrow_mut();
                    table.insert(name.clone(), binding);
                }
                match value {
                    Ok(expr) => self.eval(expr),
                    Err(value) => Ok(value),
                }
            }
            _ => {
                // Lexical
                let mut names = HashMap::new();
                names.insert(name.to_owned(), binding);
                let env = Env::from_parts(self.foo, names, Some(self.frame.clone()), None);
                match value {
                    Ok(expr) => env.eval(expr),
                    Err(value) => Ok(value),
                }
            }
        }
    }

    fn eval_block(&self, params: &Vec<Var>, body: &Expr, rtype: &Option<String>) -> Eval {
        let mut args = vec![];
        for p in params {
            let vt = match &p.typename {
                None => None,
                Some(name) => {
                    Some(self.find_class(name, p.span.clone())?.class().instance_vtable.clone())
                }
            };
            args.push(Arg::new(p.span.clone(), p.name.clone(), vt));
        }
        self.foo.make_closure(self.frame.clone(), args, body.clone(), rtype)
    }

    fn eval_cascade(&self, receiver: &Box<Expr>, chains: &Vec<Vec<Message>>) -> Eval {
        let receiver = self.eval(receiver)?;
        let mut res = receiver.clone();
        for messages in chains {
            res = self.eval_sends(receiver.clone(), messages)?;
        }
        Ok(res)
    }

    fn eval_class_definition(&self, definition: &ClassDefinition) -> Eval {
        // FIXME: allow anonymous classes
        if self.frame.parent().is_some() {
            return Unwind::error_at(definition.span.clone(), "Class definition not at toplevel");
        }
        let name = &definition.name;
        let class = self.foo.make_class(definition)?;
        self.foo.globals.borrow_mut().insert(name.to_string(), class.clone());
        Ok(class)
    }

    fn eval_eq(&self, left: &Expr, right: &Expr) -> Eval {
        if self.eval(left) == self.eval(right) {
            Ok(self.foo.make_boolean(true))
        } else {
            Ok(self.foo.make_boolean(false))
        }
    }

    fn eval_global(&self, global: &Global) -> Eval {
        match self.foo.globals.borrow().get(&global.name) {
            Some(obj) => Ok(obj.clone()),
            None => Unwind::error_at(global.span.clone(), "Undefined global"),
        }
    }

    fn eval_literal(&self, literal: &Literal) -> Eval {
        match literal {
            Literal::Boolean(value) => Ok(self.foo.make_boolean(*value)),
            Literal::Integer(value) => Ok(self.foo.make_integer(*value)),
            Literal::Float(value) => Ok(self.foo.make_float(*value)),
            Literal::String(value) => Ok(self.foo.make_string(value)),
        }
    }

    fn eval_import(&self, import: &Import) -> Eval {
        unimplemented!("eval_import")
        /* Sketch:
            - import.load_module() is responsible for adding prefixes and such
              (or eliding them or only bringing in one object, etc)

            let names = import.load_module(self.foo);
            let value = match &import.body {
                 None => Const::new(self.foo.make_string(&import.name))
                 Some(expr) => expr,
            };
            match self.foo.workspace {
              Some(ref workspace) if self.frame.receiver().is_none() => {
                 // toplevel import in workspace (not module?! how do I know?)
                 {
                   let mut table = workspace.borrow_mut();
                   table.insert_all(names);
                 }
                 self.eval(value),
              }
              _ => {
                // Lexical
                let env = Env::from_parts(self.foo, names, Some(self.frame.clone()), None);
                env.eval(value)
              }
            }
        */
    }

    fn eval_return(&self, ret: &Return) -> Eval {
        match self.frame.home() {
            None => Unwind::error_at(ret.span.clone(), "No method to return from"),
            Some(frame) => Unwind::return_from(frame, self.eval(&ret.value)?),
        }
    }

    fn eval_sends(&self, mut receiver: Object, messages: &Vec<Message>) -> Eval {
        for message in messages {
            let mut values = Vec::new();
            for arg in &message.args {
                values.push(self.eval(arg)?);
            }
            receiver = receiver.send(message.selector.as_str(), &values[..], &self.foo)?
        }
        return Ok(receiver);
    }

    fn eval_chain(&self, receiver: &Box<Expr>, messages: &Vec<Message>) -> Eval {
        self.eval_sends(self.eval(receiver)?, messages)
    }

    fn eval_seq(&self, exprs: &Vec<Expr>) -> Eval {
        // FIXME: false or nothing
        let mut result = self.foo.make_integer(0);
        for expr in exprs {
            result = self.eval(expr)?;
        }
        Ok(result)
    }

    fn find_class(&self, name: &str, span: Span) -> Eval {
        self.foo.find_class(name, span)
    }

    fn eval_typecheck(&self, expr: &Expr, typename: &str) -> Eval {
        let value = self.eval(expr)?;
        // FIXME: Wrong span.
        let class = self.find_class(typename, expr.span())?.class();
        if class.instance_vtable == value.vtable {
            Ok(value)
        } else {
            Unwind::type_error_at(expr.span(), value, class.instance_vtable.name.clone())
        }
    }

    fn eval_assign(&self, assign: &Assign) -> Eval {
        let value = self.eval(&assign.value)?;
        match self.frame.set(&assign.name, value.clone()) {
            Some(res) => res.source(&assign.span),
            None => {
                if let Some(receiver) = self.frame.receiver() {
                    if let Some(slot) = receiver.vtable.slots.get(&assign.name) {
                        return write_instance_variable(receiver, slot, value).source(&assign.span);
                    }
                } else {
                    // Not inside a method, so let's check workspace.
                    // FIXME: Are closures allowed to see into workspace?
                    if let Some(workspace) = &self.foo.workspace {
                        if let Some(binding) = workspace.borrow_mut().get_mut(&assign.name) {
                            return binding.assign(value);
                        }
                    }
                }
                Unwind::error_at(assign.span.clone(), "Cannot assign to an unbound variable")
            }
        }
    }

    fn eval_var(&self, var: &Var) -> Eval {
        if &var.name == "self" {
            match self.frame.receiver() {
                None => Unwind::error_at(var.span.clone(), "self outside method context"),
                Some(receiver) => Ok(receiver.clone()),
            }
        } else {
            match self.frame.get(&var.name) {
                Some(value) => return Ok(value),
                None => {
                    if let Some(receiver) = self.frame.receiver() {
                        if let Some(slot) = receiver.vtable.slots.get(&var.name) {
                            return read_instance_variable(receiver, slot.index);
                        }
                    } else {
                        // Not inside a method, so let's check workspace.
                        // FIXME: Are closures allowed to see into workspace?
                        if let Some(workspace) = &self.foo.workspace {
                            if let Some(binding) = workspace.borrow().get(&var.name) {
                                return Ok(binding.value.clone());;
                            }
                        }
                    }
                }
            }
            Unwind::error_at(var.span.clone(), "Unbound variable")
        }
    }
}

pub fn apply(
    receiver: Option<&Object>,
    closure: &Closure,
    call_args: &[Object],
    foo: &Foolang,
) -> Eval {
    let mut args = HashMap::new();
    for (arg, obj) in closure.params.iter().zip(call_args.into_iter().map(|x| (*x).clone())) {
        let binding = match &arg.vtable {
            None => Binding::untyped(obj),
            Some(vtable) => {
                if vtable != &obj.vtable {
                    return Unwind::type_error_at(arg.span.clone(), obj, vtable.name.clone());
                }
                Binding::typed(vtable.to_owned(), obj.to_owned())
            }
        };
        args.insert(arg.name.clone(), binding);
    }
    let env = Env::from_parts(foo, args, closure.env(), receiver.map(|x| x.clone()));
    let result = match env.eval(&closure.body) {
        Err(Unwind::ReturnFrom(ref frame, ref value)) if frame == &env.frame => value.clone(),
        Ok(value) => value,
        Err(unwind) => return Err(unwind),
    };
    if let Some(vtable) = &closure.return_vtable {
        if &result.vtable != vtable {
            return Unwind::type_error(result, vtable.name.clone()).source(&closure.body.span());
        }
    }
    Ok(result)
}

pub fn eval_all(foo: &Foolang, source: &str) -> Eval {
    let env = Env::new(foo);
    let mut parser = Parser::new(source);
    loop {
        let expr = match parser.parse() {
            Err(unwind) => {
                return Err(unwind.with_context(source));
            }
            Ok(expr) => expr,
        };
        let object = match env.eval(&expr) {
            Err(unwind) => {
                return Err(unwind.with_context(source));
            }
            Ok(object) => object,
        };
        if parser.at_eof() {
            return Ok(object);
        }
    }
}

#[cfg(test)]
pub mod utils {

    use crate::eval::*;

    pub fn eval_exception(source: &str) -> (Unwind, Foolang) {
        let foo = Foolang::new();
        match eval_all(&foo, source) {
            Err(unwind) => match &unwind {
                Unwind::Exception(..) => (unwind, foo),
                _ => panic!("Expected exception, got: {:?}", unwind),
            },
            Ok(value) => panic!("Expected exception, got: {:?}", value),
        }
    }

    pub fn eval_str(source: &str) -> Eval {
        let foo = Foolang::new();
        eval_all(&foo, source)
    }

    pub fn eval_obj(source: &str) -> (Object, Foolang) {
        let foo = Foolang::new();
        match eval_all(&foo, source) {
            Err(unwind) => panic!("Unexpected unwind:\n{:?}", unwind),
            Ok(obj) => (obj, foo),
        }
    }

    pub fn eval_ok(source: &str) -> Object {
        match eval_str(source) {
            Ok(obj) => obj,
            Err(Unwind::Exception(error, location)) => {
                panic!("Exception in eval_ok: {}:\n{}", error.what(), location.context());
            }
            Err(Unwind::ReturnFrom(..)) => panic!("Unexpected return-from in eval_ok"),
        }
    }

}
