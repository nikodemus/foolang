use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::objects::{
    read_instance_variable, write_instance_variable, Arg, Datum, Eval, Foolang, Object, Source,
    Vtable,
};
use crate::parse::{
    Array, Assign, ClassDefinition, ClassExtension, Expr, Global, Import, Literal, Message, Parser,
    Return, Var,
};
use crate::tokenstream::Span;
use crate::unwind::Unwind;

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
    pub fn typed(vtable: Rc<Vtable>, init: Object) -> Binding {
        Binding {
            vtable: Some(vtable),
            value: init,
        }
    }
    pub fn assign(&mut self, value: Object) -> Eval {
        if let Some(vtable) = &self.vtable {
            if &value.vtable != vtable {
                return Unwind::type_error(value, vtable.name.clone());
            }
        }
        self.value = value.clone();
        Ok(value)
    }
}

type SymbolTable = HashMap<String, Binding>;

/// Underlying lexical environment: most methods operate on `Env`
/// instead.
#[derive(Debug)]
pub struct EnvImpl {
    /// For debugging purposes only: number of enclosing scopes.
    depth: u32,
    /// Names defined in this scope.
    symbols: SymbolTable,
    /// Enclosing lexical environment. None for the toplevel builtin environment.
    parent: Option<Env>,
    /// Environment of the outermost lexically enclosing closure. Used to identify
    /// identify correct frame to return from.
    home: Option<Env>,
    /// Current receiver.
    receiver: Option<Object>,
}

/// Lexical environment. Exists around `EnvImpl` to provide access to
/// the `Rc`.
#[derive(Clone)]
pub struct Env {
    /// Underlying environment.
    rc: Rc<RefCell<EnvImpl>>,
    /// Vtables for builtin classes.
    pub foo: Rc<Foolang>,
}

impl PartialEq for Env {
    // Two Env are eq if they point to the same RefCell.
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.rc, &*other.rc)
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#<Env {}>", self.rc.borrow().depth)
    }
}

impl Env {
    /// Creates a new environment containing only builtins and the current directory
    /// as a relative module root.
    #[cfg(test)]
    pub fn new() -> Env {
        Env::from(Foolang::here())
    }

    /// Creates a new environment containing only builtins using
    /// the provided `Foolang` object.
    pub fn from(foo: Foolang) -> Env {
        let foorc = Rc::new(foo);
        let mut builtins = Env {
            rc: Rc::new(RefCell::new(EnvImpl {
                depth: 0,
                symbols: HashMap::new(),
                parent: None,
                home: None,
                receiver: None,
            })),
            foo: foorc.clone(),
        };
        foorc.init_env(&mut builtins);
        Env {
            rc: Rc::new(RefCell::new(EnvImpl {
                depth: 1,
                symbols: HashMap::new(),
                parent: Some(builtins),
                home: None,
                receiver: None,
            })),
            foo: foorc,
        }
    }
    /// Returns true iff underlying `EnvImpl` has no parents, implying that
    /// that this is the builtin environment.
    fn is_builtin(&self) -> bool {
        match &self.rc.borrow().parent {
            Some(_) => false,
            None => true,
        }
    }
    /// Returns true iff underlying `EnvImpl` has exactly one parent, implying
    /// that this is a toplevel environment.
    fn is_toplevel(&self) -> bool {
        match &self.rc.borrow().parent {
            Some(env) => env.is_builtin(),
            None => false,
        }
    }
    /// Returns true iff the specified name is defined in this or parent
    /// environment.
    fn has_definition(&self, name: &str) -> bool {
        let env = self.rc.borrow();
        if env.symbols.get(name).is_some() {
            true
        } else {
            match &env.parent {
                None => false,
                Some(parent_env) => parent_env.has_definition(name),
            }
        }
    }
    /// Returns the receiver of the underlying `EnvImpl`.
    fn receiver(&self) -> Option<Object> {
        let env = self.rc.borrow();
        match &env.receiver {
            Some(receiver) => Some(receiver.clone()),
            None => match &env.parent {
                Some(parent) => parent.receiver(),
                None => None,
            },
        }
    }
    /// Returns the home of the underlying `EnvImpl`.
    fn home(&self) -> Option<Env> {
        let env = self.rc.borrow();
        match &env.home {
            Some(home) => Some(home.clone()),
            None => match &env.parent {
                Some(parent) => parent.home(),
                None => None,
            },
        }
    }
    fn parent(&self) -> Env {
        self.rc.borrow().parent.clone().unwrap()
    }

    /// Creates a new environment enclosed by this one, with no additional bindings.
    /// Used to go from toplevel to not-toplevel.
    fn enclose(&self) -> Env {
        let symbols = HashMap::new();
        Env {
            rc: Rc::new(RefCell::new(EnvImpl {
                depth: self.rc.borrow().depth + 1,
                symbols,
                parent: Some(self.clone()),
                home: None,
                receiver: None,
            })),
            foo: self.foo.clone(),
        }
    }

    /// Creates a new environment enclosed by this one, containing one additional
    /// binding.
    fn bind(&self, name: &str, binding: Binding) -> Env {
        let mut symbols = HashMap::new();
        symbols.insert(String::from(name), binding);
        Env {
            rc: Rc::new(RefCell::new(EnvImpl {
                depth: self.rc.borrow().depth + 1,
                symbols,
                parent: Some(self.clone()),
                home: None,
                receiver: None,
            })),
            foo: self.foo.clone(),
        }
    }
    /// Creates a new environment enclosed by this one, containing
    /// `symbols`, `receiver`, and `closure` as the home.
    pub fn extend(&self, symbols: SymbolTable, receiver: Option<&Object>) -> Env {
        let env = Env {
            rc: Rc::new(RefCell::new(EnvImpl {
                depth: self.rc.borrow().depth + 1,
                symbols,
                parent: Some(self.clone()),
                home: self.home(),
                receiver: receiver.map(|obj| obj.clone()),
            })),
            foo: self.foo.clone(),
        };
        // If there was no lexically enclosing call environment, then this is the one.
        if env.rc.borrow().home.is_none() {
            env.rc.borrow_mut().home = Some(env.clone());
        };
        env
    }

    pub fn define(&self, name: &str, value: Object) {
        self.rc.borrow_mut().symbols.insert(String::from(name), Binding::untyped(value));
    }

    pub fn add_binding(&self, name: &str, binding: Binding) -> Env {
        // println!("add binding: {}", name);
        self.rc.borrow_mut().symbols.insert(String::from(name), binding);
        self.clone()
    }

    pub fn set(&self, name: &str, value: Object) -> Option<Eval> {
        let mut env = self.rc.borrow_mut();
        match env.symbols.get_mut(name) {
            Some(binding) => Some(binding.assign(value)),
            None => match &env.parent {
                Some(parent) => parent.set(name, value),
                None => None,
            },
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        let env = self.rc.borrow();
        match env.symbols.get(name) {
            Some(binding) => return Some(binding.value.clone()),
            None => match &env.parent {
                Some(parent) => parent.get(name),
                None => None,
            },
        }
    }

    pub fn eval_all(&self, source: &str) -> Eval {
        let mut parser = Parser::new(source, self.foo.root());
        loop {
            let expr = match parser.parse() {
                Err(unwind) => {
                    return Err(unwind.with_context(source));
                }
                Ok(expr) => expr,
            };
            let object = match self.eval(&expr) {
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

    pub fn eval(&self, expr: &Expr) -> Eval {
        use Expr::*;
        match expr {
            Array(array) => self.eval_array(array),
            Assign(assign) => self.eval_assign(assign),
            Bind(name, typename, value, body) => self.eval_bind(name, typename, value, body),
            Block(_, params, body, rtype) => self.eval_block(params, body, rtype),
            Cascade(receiver, chains) => self.eval_cascade(receiver, chains),
            ClassDefinition(definition) => self.eval_class_definition(definition),
            ClassExtension(extension) => self.eval_class_extension(extension),
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

    fn eval_array(&self, array: &Array) -> Eval {
        let mut data = Vec::new();
        let array_env = self.enclose();
        for elt in &array.data {
            data.push(array_env.eval(elt)?);
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
        // FIXME: there used to be workspace stuff there to handle 'toplevel lets'.
        // Lexical
        let env = if self.is_toplevel() {
            // FIXME: should check if the toplevel is a "workspace" or not.
            self.add_binding(name, binding)
        } else {
            self.bind(name, binding)
        };
        match value {
            Ok(expr) => env.eval(expr),
            Err(value) => Ok(value),
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
        self.foo.make_closure(self.clone(), args, body.clone(), rtype)
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
        // println!("CLASS env: {:?}", self);
        // FIXME: allow anonymous classes
        if !self.is_toplevel() {
            return Unwind::error_at(definition.span.clone(), "Class definition not at toplevel");
        }
        let name = &definition.name;
        if self.has_definition(name) {
            return Unwind::error_at(definition.span.clone(), "Cannot redefine");
        }
        let class = self.foo.make_class(definition, self)?;
        self.define(name, class.clone());
        Ok(class)
    }

    fn eval_class_extension(&self, extension: &ClassExtension) -> Eval {
        if !self.is_toplevel() {
            return Unwind::error_at(extension.span.clone(), "Class extension not at toplevel");
        }
        let class = self.find_class(&extension.name, extension.span.clone())?;
        class.extend_class(extension, self)
    }

    fn eval_eq(&self, left: &Expr, right: &Expr) -> Eval {
        if self.eval(left) == self.eval(right) {
            Ok(self.foo.make_boolean(true))
        } else {
            Ok(self.foo.make_boolean(false))
        }
    }

    // FIXME: half duplicates find_global
    pub fn find_class(&self, name: &str, span: Span) -> Eval {
        match self.find_global(name) {
            None => Unwind::error_at(span, "Undefined class"),
            Some(obj) => match &obj.datum {
                Datum::Class(_) => Ok(obj),
                _ => Unwind::error_at(span, "Not a class name"),
            },
        }
    }

    pub fn find_global(&self, name: &str) -> Option<Object> {
        if self.is_toplevel() {
            self.get(name)
        } else {
            self.parent().find_global(name)
        }
    }

    pub fn find_vtable_if_name(
        &self,
        name: &Option<String>,
        span: Span,
    ) -> Result<Option<Rc<Vtable>>, Unwind> {
        match name {
            None => Ok(None),
            Some(name) => Ok(Some(self.find_class(name, span)?.class().instance_vtable.clone())),
        }
    }

    fn eval_global(&self, global: &Global) -> Eval {
        match self.find_global(&global.name) {
            Some(obj) => Ok(obj),
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
        let module = self.foo.load_module(&import.path)?;
        let symbols = &module.rc.borrow().symbols;
        match &import.name {
            None => {
                // everything native to module, using prefix
                for (name, _) in symbols.iter() {
                    if name.contains(".") {
                        continue;
                    }
                    let alias = format!("{}.{}", &import.prefix, name);
                    if self.has_definition(&alias) {
                        return Unwind::error_at(
                            import.span.clone(),
                            &format!("Name conflict: {} already defined", &alias),
                        );
                    }
                }
                for (name, binding) in symbols.iter() {
                    if name.contains(".") {
                        continue;
                    }
                    let alias = format!("{}.{}", &import.prefix, name);
                    self.add_binding(&alias, binding.clone());
                }
            }
            Some(name) if name == "*" => {
                // everything, without prefix
                for (name, _) in symbols.iter() {
                    if name.contains(".") {
                        continue;
                    }
                    if self.has_definition(&name) {
                        return Unwind::error_at(
                            import.span.clone(),
                            &format!("Name conflict: {} already defined", &name),
                        );
                    }
                }
                for (name, binding) in symbols.iter() {
                    if name.contains(".") {
                        continue;
                    }
                    self.add_binding(&name, binding.clone());
                }
            }
            Some(name) => {
                // just this
                if self.has_definition(&name) {
                    return Unwind::error_at(
                        import.span.clone(),
                        &format!("Name conflict: {} already defined", &name),
                    );
                }
                match symbols.get(name.as_str()) {
                    None => {
                        return Unwind::error_at(
                            import.span.clone(),
                            &format!("Cannot import {}: not defined in module", &name),
                        )
                    }
                    Some(binding) => {
                        self.add_binding(&name, binding.clone());
                    }
                }
            }
        }
        match import.body {
            // FIXME: Should be path + name
            None => Ok(self.foo.make_string(&import.path.to_string_lossy())),
            Some(ref expr) => self.eval(expr),
        }
    }

    fn eval_return(&self, ret: &Return) -> Eval {
        match self.home() {
            None => Unwind::error_at(ret.span.clone(), "Nothing to return from"),
            Some(env) => Unwind::return_from(env, self.eval(&ret.value)?),
        }
    }

    fn eval_sends(&self, mut receiver: Object, messages: &Vec<Message>) -> Eval {
        for message in messages {
            let mut values = Vec::new();
            for arg in &message.args {
                values.push(self.eval(arg)?);
            }
            receiver = receiver.send(message.selector.as_str(), &values[..], &self)?
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
        match self.set(&assign.name, value.clone()) {
            Some(res) => res.source(&assign.span),
            None => {
                if let Some(receiver) = self.receiver() {
                    if let Some(slot) = receiver.vtable.slots.get(&assign.name) {
                        return write_instance_variable(&receiver, slot, value)
                            .source(&assign.span);
                    }
                }
                // FIXME: there used to be a workspace lookup here...
                Unwind::error_at(assign.span.clone(), "Cannot assign to an unbound variable")
            }
        }
    }

    fn eval_var(&self, var: &Var) -> Eval {
        if &var.name == "self" {
            match self.receiver() {
                None => Unwind::error_at(var.span.clone(), "self outside method context"),
                Some(receiver) => Ok(receiver.clone()),
            }
        } else {
            match self.get(&var.name) {
                Some(value) => return Ok(value),
                None => {
                    if let Some(receiver) = self.receiver() {
                        if let Some(slot) = receiver.vtable.slots.get(&var.name) {
                            return read_instance_variable(&receiver, slot.index);
                        }
                    }
                    /*
                    println!(
                        "UNBOUND: {:?}\n    ENV: {:?}\n    REC: {:?}",
                        &var,
                        self.rc.borrow(),
                        self.receiver()
                    );
                     */
                    // FIXME: There used to be workspace handling here
                }
            }
            Unwind::error_at(var.span.clone(), "Unbound variable")
        }
    }
}

#[cfg(test)]
pub mod utils {

    use crate::eval::*;

    pub fn eval_exception(source: &str) -> (Unwind, Env) {
        let env = Env::new();
        match env.eval_all(source) {
            Err(unwind) => match &unwind {
                Unwind::Exception(..) => (unwind, env),
                _ => panic!("Expected exception, got: {:?}", unwind),
            },
            Ok(value) => panic!("Expected exception, got: {:?}", value),
        }
    }

    pub fn eval_str(source: &str) -> Eval {
        let env = Env::new();
        env.eval_all(source)
    }

    pub fn eval_obj(source: &str) -> (Object, Env) {
        let env = Env::new();
        match env.eval_all(source) {
            Err(unwind) => panic!("Unexpected unwind:\n{:?}", unwind),
            Ok(obj) => (obj, env),
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
