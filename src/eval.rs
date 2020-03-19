use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::path::{Path};

use crate::objects::{
    read_instance_variable, write_instance_variable, Arg, Class, Datum, Eval, Foolang, Object,
    Source, Vtable
};
use crate::syntax::Syntax;
use crate::expr::*;
use crate::def::*;
use crate::parse::Parser;
use crate::span::Span;
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
            value.typecheck(vtable)?;
        }
        self.value = value.clone();
        Ok(value)
    }
}

type SymbolTable = HashMap<String, Binding>;

/// Underlying lexical environment: most methods operate on `Env`
/// or EnvRef instead.
#[derive(Debug)]
pub struct EnvFrame {
    /// For debugging purposes only: number of enclosing scopes.
    depth: u32,
    /// Names defined in this scope.
    symbols: SymbolTable,
    /// Enclosing lexical environment. None for the toplevel builtin environment.
    parent: Option<EnvRef>,
    /// Environment of the outermost lexically enclosing closure. Used to identify
    /// identify correct frame to return from.
    home: Option<EnvRef>,
    /// Current receiver.
    receiver: Option<Object>,
}

#[derive(Debug, Clone)]
pub struct EnvRef {
    frame: Rc<RefCell<EnvFrame>>,
}

impl PartialEq for EnvRef {
    // Two EnvRefs are eq if they point to the same frame.
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.frame, &*other.frame)
    }
}

impl EnvRef {
    pub fn new() -> EnvRef {
        EnvRef {
            frame: Rc::new(RefCell::new(EnvFrame {
                depth: 0,
                symbols: HashMap::new(),
                parent: None,
                home: None,
                receiver: None,
            })),
        }
    }
    pub fn enclose(&self) -> EnvRef {
        EnvRef {
            frame: Rc::new(RefCell::new(EnvFrame {
                depth: self.depth() + 1,
                symbols: HashMap::new(),
                parent: Some(self.clone()),
                home: None,
                receiver: None,
            })),
        }
    }
    fn extend(&self, symbols: SymbolTable, receiver: Option<&Object>) -> EnvRef {
        let env_ref = EnvRef {
            frame: Rc::new(RefCell::new(EnvFrame {
                depth: self.depth() + 1,
                symbols,
                parent: Some(self.clone()),
                home: self.home(),
                receiver: receiver.map(|obj| obj.clone()),
            })),
        };
        // If there was no lexically enclosing call environment, then this is
        // the one.
        if env_ref.home().is_none() {
            env_ref.frame.borrow_mut().home = Some(env_ref.clone());
        }
        env_ref
    }
    fn depth(&self) -> u32 {
        self.frame.borrow().depth
    }
    fn parent(&self) -> Option<EnvRef> {
        self.frame.borrow().parent.clone()
    }
    fn is_toplevel(&self) -> bool {
        self.depth() <= 1
    }
    fn has_definition(&self, name: &str) -> bool {
        let frame = self.frame.borrow();
        match frame.symbols.get(name) {
            Some(_) => true,
            None => match &frame.parent {
                None => false,
                Some(parent) => parent.has_definition(name),
            },
        }
    }
    fn receiver(&self) -> Option<Object> {
        let frame = self.frame.borrow();
        match &frame.receiver {
            Some(receiver) => Some(receiver.clone()),
            None => match &frame.parent {
                Some(parent) => parent.receiver(),
                None => None,
            },
        }
    }
    fn home(&self) -> Option<EnvRef> {
        let frame = self.frame.borrow();
        match &frame.home {
            Some(home) => Some(home.clone()),
            None => match &frame.parent {
                Some(parent) => parent.home(),
                None => None,
            },
        }
    }
    fn add_binding(&self, name: &str, binding: Binding) {
        self.frame.borrow_mut().symbols.insert(String::from(name), binding);
    }
    pub fn define(&self, name: &str, value: Object) {
        self.add_binding(name, Binding::untyped(value));
    }
    fn set(&self, name: &str, value: Object) -> Option<Eval> {
        let mut frame = self.frame.borrow_mut();
        match frame.symbols.get_mut(name) {
            Some(binding) => Some(binding.assign(value)),
            None => match &frame.parent {
                Some(parent) => parent.set(name, value),
                None => None,
            },
        }
    }
    fn get(&self, name: &str) -> Option<Object> {
        match self.get_binding(name) {
            None => None,
            Some(binding) => return Some(binding.value),
        }
    }
    fn get_binding(&self, name: &str) -> Option<Binding> {
        let frame = self.frame.borrow();
        match frame.symbols.get(name) {
            Some(binding) => return Some(binding.clone()),
            None => match &frame.parent {
                Some(parent) => parent.get_binding(name),
                None => None,
            },
        }
    }
    pub fn find_global(&self, name: &str) -> Option<Object> {
        if self.is_toplevel() {
            self.get(name)
        } else {
            // NOTE: only time parent does not exist is the builtin env,
            // which goes to leg above -- ergo this unwrap is safe.
            self.parent().unwrap().find_global(name)
        }
    }
    pub fn import_name(&self, module: &EnvRef, name: &str) -> Result<(), Unwind> {
        match module.get_binding(name) {
            None => {
                return Unwind::error(&format!("Cannot import {}: not defined in module", &name))
            }
            Some(binding) => {
                if let Some(old_binding) = self.get_binding(&name) {
                    if &old_binding != &binding {
                        return Unwind::error(&format!("Name conflict: {} already defined", &name));
                    }
                }
                self.add_binding(&name, binding.clone());
            }
        }
        Ok(())
    }
    pub fn import_everything(&self, module: &EnvRef) -> Result<(), Unwind> {
        let mut todo = vec![];
        for (name, binding) in module.frame.borrow().symbols.iter() {
            if name.contains(".") {
                continue;
            }
            if let Some(old_binding) = self.get_binding(&name) {
                if &old_binding != binding {
                    return Unwind::error(&format!("Name conflict: {} already defined", &name));
                }
            } else {
                todo.push((name.to_string(), binding.clone()));
            }
        }
        for (name, binding) in todo.into_iter() {
            self.add_binding(&name, binding);
        }
        Ok(())
    }
    pub fn import_prefixed(&self, module: &EnvRef, prefix: &str) -> Result<(), Unwind> {
        let mut todo = vec![];
        for (name, binding) in module.frame.borrow().symbols.iter() {
            if name.contains(".") {
                continue;
            }
            let alias = format!("{}.{}", prefix, name);
            if let Some(old_binding) = self.get_binding(&alias) {
                if &old_binding != binding {
                    return Unwind::error(&format!("Name conflict: {} already defined", &name));
                }
            } else {
                todo.push((alias, binding.clone()));
            }
        }
        for (name, binding) in todo.into_iter() {
            self.add_binding(&name, binding);
        }
        Ok(())
    }
}

/// Lexical environment. Exists around `EnvImpl` to provide access to
/// the `Rc`.
#[derive(Clone)]
pub struct Env {
    /// Underlying environment.
    pub env_ref: EnvRef,
    /// Vtables for builtin classes.
    pub foo: Rc<Foolang>,
}

impl PartialEq for Env {
    // Two Env are eq if they point to the same EnvRef.
    fn eq(&self, other: &Self) -> bool {
        self.env_ref.eq(&other.env_ref)
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#<Env {}>", self.env_ref.depth())
    }
}

impl Env {
    /// Creates a new environment containing only builtins and the current directory
    /// as a relative module root.
    #[cfg(test)]
    pub fn new() -> Env {
        Foolang::here().toplevel_env()
    }

    pub fn load_code<P: AsRef<Path>>(self, code: &str, root: P) -> Result<Env, Unwind> {
        let mut parser = Parser::new(&code, root);
        while !parser.at_eof() {
            match parser.parse() {
                Ok(Syntax::Def(def)) => self.augment(&def).context(&code)?,
                // FIXME: Better error needed here.
                Ok(Syntax::Expr(_)) => return Unwind::error("Expression at toplevel!"),
                Err(unwind) => return Err(unwind.with_context(&code)),
            };
        }
        Ok(self)
    }

    /// Returns true iff underlying `EnvImpl` has is child of the builtin environment.
    /// or the builtin environment.
    fn is_toplevel(&self) -> bool {
        self.env_ref.is_toplevel()
    }
    /// Returns true iff the specified name is defined in this or parent
    /// environment.
    fn has_definition(&self, name: &str) -> bool {
        self.env_ref.has_definition(name)
    }
    /// Returns the receiver of the underlying `EnvFrame`.
    fn receiver(&self) -> Option<Object> {
        self.env_ref.receiver()
    }
    /// Returns the home of the underlying `EnvFrame`.
    fn home(&self) -> Option<EnvRef> {
        self.env_ref.home()
    }

    /// Creates a new environment enclosed by this one, with no additional bindings.
    /// Used to go from toplevel to not-toplevel.
    fn enclose(&self) -> Env {
        Env {
            env_ref: self.env_ref.enclose(),
            foo: self.foo.clone(),
        }
    }

    /// Creates a new environment enclosed by this one, containing one additional
    /// binding.
    fn bind(&self, name: &str, binding: Binding) -> Env {
        let child = self.enclose();
        child.add_binding(name, binding);
        child
    }
    /// Creates a new environment enclosed by this one, containing
    /// `symbols`, `receiver`, and `closure` as the home.
    pub fn extend(&self, symbols: SymbolTable, receiver: Option<&Object>) -> Env {
        Env {
            env_ref: self.env_ref.extend(symbols, receiver),
            foo: self.foo.clone(),
        }
    }

    pub fn define(&self, name: &str, value: Object) {
        self.env_ref.define(name, value);
    }

    pub fn add_binding(&self, name: &str, binding: Binding) {
        // println!("add binding: {}", name);
        self.env_ref.add_binding(name, binding);
    }

    pub fn set(&self, name: &str, value: Object) -> Option<Eval> {
        self.env_ref.set(name, value)
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.env_ref.get(name)
    }

    // FIXME: Should probably be called "load_workspace"
    pub fn eval_all(&self, source: &str) -> Eval {
        let mut parser = Parser::new(source, self.foo.root());
        let env = self.clone();
        let mut res = self.foo.make_boolean(false);
        while !parser.at_eof() {
            res = match parser.parse() {
                Ok(Syntax::Def(def)) => env.augment(&def).context(&source)?,
                Ok(Syntax::Expr(expr)) => env.eval(&expr).context(&source)?,
                Err(unwind) => return Err(unwind.with_context(&source)),
            };
        }
        Ok(res)
    }

    pub fn augment(&self, def: &Def) -> Eval {
        match def {
            Def::ClassDef(klass) => self.do_class(klass),
            Def::DefineDef(def) => self.do_define(def),
            Def::ExtensionDef(extension) => self.do_extension(extension),
            Def::ImportDef(import) => self.do_import(import),
            Def::InterfaceDef(interface) => self.do_interface(interface),
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
            Const(constant) => self.eval_constant(constant),
            Dictionary(dictionary) => self.eval_dictionary(dictionary),
            Eq(eq) => self.eval_eq(eq),
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
        let value = self.eval(&bind.value)?;
        let binding = match bind.typename {
            None => Binding::untyped(value),
            Some(ref typename) => {
                let vt = self.find_type(typename)?;
                value.typecheck(&vt).source(&bind.value.span())?;
                // FIXME: make the typecheck explicit
                Binding::typed(vt, value)
            }
        };
        let tmp = binding.value.clone();
        // FIXME: there used to be workspace stuff there to handle 'toplevel lets'.
        let env = if self.is_toplevel() {
            // FIXME: should check if the toplevel is a "workspace" or not.
            self.add_binding(&bind.name, binding);
            self.clone()
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
        let mut parameter_types = vec![];
        for p in &block.params {
            args.push(Arg::new(p.span.clone(), p.name.clone()));
            parameter_types.push(&p.typename);
        }
        self.foo.make_closure(
            self.clone(),
            args,
            (*block.body).clone(),
            parameter_types,
            &block.rtype,
        )
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
        if self.is_toplevel() {
            Ok(())
        } else {
            Unwind::error_at(span.clone(), &format!("{} not at toplevel", what))
        }
    }

    fn check_not_defined(&self, name: &str, span: &Span, what: &str) -> Result<(), Unwind> {
        if self.has_definition(name) {
            return Unwind::error_at(span.clone(), &format!("Cannot redefine {}", what));
        };
        Ok(())
    }

    fn do_class(&self, definition: &ClassDef) -> Eval {
        // println!("CLASS env: {:?}", self);
        // FIXME: allow anonymous classes
        self.check_toplevel(&definition.span, "Class definition")?;
        let name = &definition.name;
        self.check_not_defined(name, &definition.span, "Class")?;
        let class = self.foo.make_class(definition, self)?;
        self.define(name, class.clone());
        Ok(class)
    }

    fn do_define(&self, definition: &DefineDef) -> Eval {
        self.check_toplevel(&definition.span, "Constant definition")?;
        let name = &definition.name;
        self.check_not_defined(name, &definition.span, "Constant")?;
        let value = self.eval(&definition.init)?;
        self.define(name, value.clone());
        Ok(value)
    }

    fn do_extension(&self, extension: &ExtensionDef) -> Eval {
        if !self.is_toplevel() {
            return Unwind::error_at(extension.span.clone(), "Extension not at toplevel");
        }
        let class = self.find_global_or_unwind(&extension.name)?;
        class.extend_class(extension, self)
    }

    fn do_import(&self, import: &ImportDef) -> Eval {
        let module = &self.foo.load_module(&import.path)?.env_ref;
        let res = match &import.name {
            None => self.env_ref.import_prefixed(&module, &import.prefix),
            Some(name) if name == "*" => self.env_ref.import_everything(&module),
            Some(name) => self.env_ref.import_name(&module, name),
        };
        if let Err(mut unwind) = res {
            unwind.add_span(&import.span);
            return Err(unwind);
        }
        Ok(self.foo.make_string(&import.path.to_string_lossy()))
    }

    fn do_interface(&self, interface: &InterfaceDef) -> Eval {
        self.check_toplevel(&interface.span, "Interface definition")?;
        let name = &interface.name;
        self.check_not_defined(&interface.name, &interface.span, "Interface")?;
        let interface = self.foo.make_interface(interface, self)?;
        self.define(name, interface.clone());
        Ok(interface)
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
        if self.eval(&eq.left)? == self.eval(&eq.right)? {
            Ok(self.foo.make_boolean(true))
        } else {
            Ok(self.foo.make_boolean(false))
        }
    }

    // NOTE: The name is correct: vtables stand in for classes right now,
    // and once we have non-vtable types this will return Option<Type>
    // instead. Ditto for maybe_type.
    pub fn maybe_type(&self, maybe_name: &Option<String>) -> Result<Option<Rc<Vtable>>, Unwind> {
        match maybe_name {
            None => return Ok(None),
            Some(name) => Ok(Some(self.find_type(name)?)),
        }
    }

    // NOTE: name is correct, see maybe_type for more.
    pub fn find_type(&self, name: &str) -> Result<Rc<Vtable>, Unwind> {
        match self.find_global(name) {
            None => Unwind::error(&format!("Undefined type: {}", name)),
            Some(obj) => match &obj.datum {
                Datum::Class(ref class) => Ok(class.instance_vtable.clone()),
                _ => Unwind::error(&format!("Not a type: {}", name)),
            },
        }
    }

    pub fn find_class(&self, name: &str) -> Result<Rc<Class>, Unwind> {
        match self.find_global(name) {
            None => Unwind::error(&format!("Undefined class: {}", name)),
            Some(obj) => match &obj.datum {
                Datum::Class(ref class) if !class.interface => Ok(class.clone()),
                _ => panic!("Interface, not class: {}", name),
                //_ => Unwind::error(&format!("Interface, not class: {}", name)),
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
        self.env_ref.find_global(name)
    }

    pub fn find_global_or_unwind(&self, name: &str) -> Eval {
        match self.find_global(name) {
            Some(obj) => Ok(obj),
            None => Unwind::error(&format!("Undefined global: {}", name)),
        }
    }

    fn eval_constant(&self, constant: &Const) -> Eval {
        match &constant.literal {
            Literal::Boolean(value) => Ok(self.foo.make_boolean(*value)),
            Literal::Integer(value) => Ok(self.foo.make_integer(*value)),
            Literal::Float(value) => Ok(self.foo.make_float(*value)),
            Literal::String(value) => Ok(self.foo.make_string(value)),
        }
    }

    pub fn import_everything(&self, module: &Env) -> Eval {
        self.env_ref.import_everything(&module.env_ref)?;
        Ok(self.foo.make_boolean(true))
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
        let expr = &typecheck.expr;
        let value = self.eval(expr)?;
        value.typecheck(&self.find_type(&typecheck.typename)?).source(&expr.span())?;
        Ok(value)
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
                        self.env_impl.borrow(),
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
