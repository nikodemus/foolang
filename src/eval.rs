use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::objects::{
    read_instance_variable, write_instance_variable, Arg, Class, Datum, Eval, Foolang, Object,
    Source, Vtable,
};
use crate::parse::{
    Array, Assign, Bind, Block, Cascade, Chain, ClassDefinition, ClassExtension, Const, Dictionary,
    Eq, Expr, Global, Import, InterfaceDefinition, Literal, Message, Parser, Raise, Return, Seq,
    Typecheck, Var,
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
            Bind(bind) => self.eval_bind(bind),
            Block(block) => self.eval_block(block),
            Cascade(cascade) => self.eval_cascade(cascade),
            ClassDefinition(definition) => self.eval_class_definition(definition),
            ClassExtension(extension) => self.eval_class_extension(extension),
            Const(constant) => self.eval_constant(constant),
            Dictionary(dictionary) => self.eval_dictionary(dictionary),
            Eq(eq) => self.eval_eq(eq),
            Global(global) => self.eval_global(global),
            Import(import) => self.eval_import(import),
            InterfaceDefinition(interface) => self.eval_interface(interface),
            Raise(raise) => self.eval_raise(raise),
            Return(ret) => self.eval_return(ret),
            Chain(chain) => self.eval_chain(chain),
            Seq(seq) => self.eval_seq(&seq),
            Typecheck(typecheck) => self.eval_typecheck(typecheck),
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

    fn eval_bind(&self, bind: &Bind) -> Eval {
        let binding = match bind.typename {
            None => Binding::untyped(self.eval(&bind.value)?),
            Some(ref typename) => {
                let class = self.find_class(typename)?;
                // FIXME: make the typecheck explicit
                Binding::typed(
                    class.instance_vtable.clone(),
                    self.do_typecheck(&bind.value, typename)?,
                )
            }
        };
        let tmp = binding.value.clone();
        // FIXME: there used to be workspace stuff there to handle 'toplevel lets'.
        let env = if self.is_toplevel() {
            // FIXME: should check if the toplevel is a "workspace" or not.
            self.add_binding(&bind.name, binding)
        } else {
            self.bind(&bind.name, binding)
        };
        match &bind.body {
            None => Ok(tmp),
            Some(body) => env.eval(&body),
        }
    }

    fn eval_block(&self, block: &Block) -> Eval {
        let mut args = vec![];
        for p in &block.params {
            let vt = match &p.typename {
                None => None,
                Some(name) => Some(self.find_class(name)?.instance_vtable.clone()),
            };
            args.push(Arg::new(p.span.clone(), p.name.clone(), vt));
        }
        self.foo.make_closure(self.clone(), args, (*block.body).clone(), &block.rtype)
    }

    fn eval_cascade(&self, cascade: &Cascade) -> Eval {
        let receiver = self.eval(&cascade.receiver)?;
        let mut res = receiver.clone();
        for messages in &cascade.chains {
            res = self.eval_sends(receiver.clone(), messages)?;
        }
        Ok(res)
    }

    fn check_toplevel(&self, span: &Span, what: &str) -> Result<(), Unwind> {
        if !self.is_toplevel() {
            return Unwind::error_at(span.clone(), &format!("{} not at toplevel", what));
        }
        Ok(())
    }

    fn check_not_defined(&self, name: &str, span: &Span, what: &str) -> Result<(), Unwind> {
        if self.has_definition(name) {
            return Unwind::error_at(span.clone(), &format!("Cannot redefine {}", what));
        };
        Ok(())
    }

    fn eval_class_definition(&self, definition: &ClassDefinition) -> Eval {
        // println!("CLASS env: {:?}", self);
        // FIXME: allow anonymous classes
        self.check_toplevel(&definition.span, "Class definition")?;
        let name = &definition.name;
        self.check_not_defined(name, &definition.span, "Class")?;
        let class = self.foo.make_class(definition, self)?;
        self.define(name, class.clone());
        Ok(class)
    }

    fn eval_class_extension(&self, extension: &ClassExtension) -> Eval {
        if !self.is_toplevel() {
            return Unwind::error_at(extension.span.clone(), "Class extension not at toplevel");
        }
        let class = self.find_global_or_unwind(&extension.name)?;
        class.extend_class(extension, self)
    }

    fn eval_dictionary(&self, dictionary: &Dictionary) -> Eval {
        //
        // We _could_ represent dictionary construction with just messages,
        // but then we would not be able to print source back from Exprs
        // felicituously.
        //
        // FIXME: bogus span
        let mut data = HashMap::new();
        for (k, v) in dictionary.assoc.iter() {
            data.insert(self.eval(k)?, self.eval(v)?);
        }
        Ok(self.foo.into_dictionary(data))
    }

    fn eval_eq(&self, eq: &Eq) -> Eval {
        if self.eval(&eq.left) == self.eval(&eq.right) {
            Ok(self.foo.make_boolean(true))
        } else {
            Ok(self.foo.make_boolean(false))
        }
    }

    // NOTE: The name is correct: vtables stand in for classes right now,
    // and once we have non-vtable types this will return Option<Type>
    // instead.
    pub fn maybe_type(&self, maybe_name: &Option<String>) -> Option<Rc<Vtable>> {
        match maybe_name {
            None => return None,
            Some(name) => {
                let class = match self.find_class(name) {
                    Err(_) => return None,
                    Ok(class) => class,
                };
                Some(class.instance_vtable.clone())
            }
        }
    }

    pub fn find_class(&self, name: &str) -> Result<Rc<Class>, Unwind> {
        match self.find_global(name) {
            None => Unwind::error(&format!("Undefined class: {}", name)),
            Some(obj) => match &obj.datum {
                Datum::Class(ref class) if !class.interface => Ok(class.clone()),
                _ => Unwind::error(&format!("Interface, not class: {}", name)),
            },
        }
    }

    pub fn find_interface(&self, name: &str) -> Result<Rc<Class>, Unwind> {
        match self.find_global(name) {
            None => Unwind::error(&format!("Undefined interface: {}", name)),
            Some(obj) => match &obj.datum {
                Datum::Class(ref class) if class.interface => Ok(class.clone()),
                _ => Unwind::error(&format!("Class, not interface: {}", name)),
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

    pub fn find_global_or_unwind(&self, name: &str) -> Eval {
        match self.find_global(name) {
            Some(obj) => Ok(obj),
            None => Unwind::error(&format!("Undefined global: {}", name)),
        }
    }

    pub fn find_vtable_if_name(&self, name: &Option<String>) -> Result<Option<Rc<Vtable>>, Unwind> {
        match name {
            None => Ok(None),
            Some(name) => Ok(Some(self.find_class(name)?.instance_vtable.clone())),
        }
    }

    fn eval_global(&self, global: &Global) -> Eval {
        self.find_global_or_unwind(&global.name).map_err(|mut err| {
            err.add_span(&global.span);
            err
        })
    }

    fn eval_constant(&self, constant: &Const) -> Eval {
        match &constant.literal {
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

    fn eval_interface(&self, interface: &InterfaceDefinition) -> Eval {
        self.check_toplevel(&interface.span, "Interface definition")?;
        let name = &interface.name;
        self.check_not_defined(&interface.name, &interface.span, "Interface")?;
        let interface = self.foo.make_interface(interface, self)?;
        self.define(name, interface.clone());
        Ok(interface)
    }

    fn eval_raise(&self, raise: &Raise) -> Eval {
        Unwind::error_at(raise.value.span(), self.eval(&raise.value)?.string_as_str())
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

    fn eval_chain(&self, chain: &Chain) -> Eval {
        self.eval_sends(self.eval(&chain.receiver)?, &chain.messages)
    }

    fn eval_seq(&self, seq: &Seq) -> Eval {
        let mut result = self.foo.make_boolean(false);
        for expr in &seq.exprs {
            result = self.eval(expr)?;
        }
        Ok(result)
    }

    fn eval_typecheck(&self, typecheck: &Typecheck) -> Eval {
        self.do_typecheck(&typecheck.expr, &typecheck.typename)
    }

    fn do_typecheck(&self, expr: &Expr, typename: &str) -> Eval {
        let value = self.eval(expr)?;
        let class = self.find_class(typename)?;
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
                    if let Some(slot) = receiver.slots().get(&assign.name) {
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
                        if let Some(slot) = receiver.slots().get(&var.name) {
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
            Unwind::error_at(var.span.clone(), &format!("Unbound variable: {}", &var.name))
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
