use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

use crate::def::*;
use crate::expr::*;
use crate::objects::{
    read_instance_variable, write_instance_variable, Arg, Datum, Eval, Foolang, Object, Source,
};
use crate::parse::Parser;
use crate::source_location::SourceLocation;
use crate::syntax::Syntax;
use crate::unwind::Unwind;

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub typed: Option<Object>,
    pub value: Object,
}

impl Binding {
    pub fn untyped(init: Object) -> Binding {
        Binding {
            typed: None,
            value: init,
        }
    }
    pub fn typed(typed: Object, init: Object, env: &Env) -> Result<Binding, Unwind> {
        let ok = typed.send("typecheck:", &[init], env)?;
        Ok(Binding {
            typed: Some(typed),
            value: ok,
        })
    }
    pub fn check_assign(&self, value: &Object, env: &Env) -> Result<(), Unwind> {
        if let Some(typed) = &self.typed {
            typed.send("typecheck:", std::slice::from_ref(value), env)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SymbolTable {
    Big(Vec<(String, Binding)>),
    Small((String, Binding)),
    Empty,
}

#[derive(Debug, PartialEq, Clone)]
enum HomeRef {
    None,
    This,
    Other(EnvRef),
}

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
    home: HomeRef,
    /// Current receiver.
    receiver: Option<Object>,
}

impl EnvFrame {
    fn ensure_here(&mut self, name: &str, binding: Binding) {
        match &mut self.symbols {
            SymbolTable::Big(vec) => {
                for entry in vec.iter_mut() {
                    if entry.0 == name {
                        entry.1 = binding;
                        return;
                    }
                }
                vec.push((String::from(name), binding));
                return;
            }
            SymbolTable::Small(pair) => {
                if pair.0 == name {
                    // KLUDGE: This is allowed due to eval_bind()'s toplevel
                    // hack.
                    pair.1 = binding;
                } else {
                    let mut vec = Vec::with_capacity(2);
                    vec.push((pair.0.to_string(), pair.1.clone()));
                    self.symbols = SymbolTable::Big(vec);
                    return self.ensure_here(name, binding);
                }
            }
            SymbolTable::Empty => {
                self.symbols = SymbolTable::Small((name.to_string(), binding));
            }
        }
    }
    fn get_here(&self, name: &str) -> Option<&Binding> {
        match &self.symbols {
            SymbolTable::Big(vec) => {
                for entry in vec.iter() {
                    if entry.0 == name {
                        return Some(&entry.1);
                    }
                }
                return None;
            }
            SymbolTable::Small((key, value)) => {
                if key == name {
                    Some(value)
                } else {
                    None
                }
            }
            SymbolTable::Empty => None,
        }
    }
    fn set_here(&mut self, name: &str, value: Object) {
        match &mut self.symbols {
            SymbolTable::Big(vec) => {
                for entry in vec.iter_mut() {
                    if entry.0 == name {
                        entry.1.value = value;
                        return;
                    }
                }
            }
            SymbolTable::Small(pair) => {
                if pair.0 == name {
                    pair.1.value = value;
                    return;
                }
            }
            _ => {
                unreachable!();
            }
        }
        unreachable!();
    }
    fn has_definition(&self, name: &str) -> bool {
        match &self.symbols {
            SymbolTable::Big(vec) => {
                for entry in vec.iter() {
                    if entry.0 == name {
                        return true;
                    }
                }
                return false;
            }
            SymbolTable::Small((key, _)) if key == name => return true,
            _ => {}
        }
        match &self.parent {
            None => false,
            Some(parent) => parent.has_definition(name),
        }
    }
    fn iter(&self) -> Box<dyn std::iter::Iterator<Item = (&String, &Binding)> + '_> {
        match &self.symbols {
            SymbolTable::Big(vec) => Box::new(vec.iter().map(|each| (&each.0, &each.1))),
            SymbolTable::Small(pair) => Box::new(std::iter::once((&pair.0, &pair.1))),
            SymbolTable::Empty => Box::new(std::iter::empty()),
        }
    }
    fn len(&self) -> usize {
        match &self.symbols {
            SymbolTable::Big(map) => map.len(),
            SymbolTable::Small(_) => 1,
            SymbolTable::Empty => 0,
        }
    }
}

// static mut DEBUG_DEPTH: usize = 0;

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
    pub fn debug(&self) {
        println!("---");
        for (k, _) in self.frame.borrow().iter() {
            println!("- {}", k);
        }
        if let Some(parent) = &self.frame.borrow().parent {
            parent.debug();
        }
    }
    pub fn new() -> EnvRef {
        EnvRef {
            frame: Rc::new(RefCell::new(EnvFrame {
                depth: 0,
                symbols: SymbolTable::Empty,
                parent: None,
                home: HomeRef::None,
                receiver: None,
            })),
        }
    }

    pub fn enclose(&self) -> EnvRef {
        EnvRef {
            frame: Rc::new(RefCell::new(EnvFrame {
                depth: self.depth() + 1,
                symbols: SymbolTable::Empty,
                parent: Some(self.clone()),
                home: HomeRef::None,
                receiver: None,
            })),
        }
    }

    pub fn extend(&self, symbols: SymbolTable, receiver: Option<&Object>) -> EnvRef {
        // If there was no lexically enclosing call environment, then this is
        // the one.
        let mut home = self.homeref();
        if home == HomeRef::None {
            home = HomeRef::This;
        }
        EnvRef {
            frame: Rc::new(RefCell::new(EnvFrame {
                depth: self.depth() + 1,
                symbols,
                parent: Some(self.clone()),
                home,
                receiver: receiver.map(|obj| obj.clone()),
            })),
        }
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
        self.frame.borrow().has_definition(name)
    }
    fn receiver(&self) -> Option<Object> {
        let frame = self.frame.borrow();
        match &frame.home {
            HomeRef::Other(home) => home.receiver(),
            _ => match &frame.receiver {
                Some(receiver) => Some(receiver.clone()),
                None => match &frame.parent {
                    Some(parent) => parent.receiver(),
                    None => None,
                },
            },
        }
    }
    fn homeref(&self) -> HomeRef {
        let frame = self.frame.borrow();
        match &frame.home {
            HomeRef::None => {
                if let Some(parent) = &frame.parent {
                    parent.homeref()
                } else {
                    HomeRef::None
                }
            }
            HomeRef::This => HomeRef::Other(self.clone()),
            _ => frame.home.clone(),
        }
    }
    fn home(&self) -> Option<EnvRef> {
        match self.homeref() {
            HomeRef::None => None,
            HomeRef::This => Some(self.clone()),
            HomeRef::Other(home) => Some(home.clone()),
        }
    }
    fn ensure_binding(&self, name: &str, binding: Binding) {
        self.frame.borrow_mut().ensure_here(name, binding);
    }
    pub fn define(&self, name: &str, value: Object) {
        self.ensure_binding(name, Binding::untyped(value));
    }
    fn set(&self, name: &str, value: Object, env: &Env) -> Option<Eval> {
        match self.frame.borrow().get_here(name) {
            Some(binding) => {
                if let Err(e) = binding.check_assign(&value, env) {
                    return Some(Err(e));
                }
            }
            None => match &self.frame.borrow().parent {
                Some(parent) => return parent.set(name, value, env),
                None => return None,
            },
        }
        self.frame.borrow_mut().set_here(name, value.clone());
        Some(Ok(value))
    }
    fn receiver_class(&self) -> Option<Object> {
        if let Some(receiver) = self.receiver() {
            receiver.vtable.class.borrow().clone()
        } else {
            None
        }
    }
    fn get(&self, name: &str) -> Option<Object> {
        if name == "self" {
            return self.receiver();
        }
        if name == "Self" {
            return self.receiver_class();
        }
        match self.get_binding(name) {
            None => None,
            Some(binding) => Some(binding.value.clone()),
        }
    }
    fn get_binding(&self, name: &str) -> Option<Binding> {
        let frame = self.frame.borrow();
        match frame.get_here(name) {
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
                self.ensure_binding(&name, binding.clone());
            }
        }
        Ok(())
    }
    pub fn import_everything(&self, module: &EnvRef) -> Result<(), Unwind> {
        let mut todo = vec![];
        for (name, binding) in module.frame.borrow().iter() {
            if name.contains(".") || name.starts_with("_") {
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
            self.ensure_binding(&name, binding);
        }
        Ok(())
    }
    pub fn import_prefixed(&self, module: &EnvRef, prefix: &str) -> Result<(), Unwind> {
        let mut todo = vec![];
        for (name, binding) in module.frame.borrow().iter() {
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
            self.ensure_binding(&name, binding);
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
                Ok(Syntax::Expr(expr)) => {
                    return Unwind::error_at(expr.source_location(), "Expression at toplevel")
                }
                Err(unwind) => return Err(unwind.with_context(&code)),
            };
        }
        Ok(self)
    }

    pub fn load_file<P: AsRef<Path>>(self, code: P, root: P) -> Result<Env, Unwind> {
        Parser::parse_file(code, root, |parser: &mut Parser| {
            while !parser.at_eof() {
                match parser.parse()? {
                    Syntax::Def(def) => self.augment(&def).context(parser.code())?,
                    // FIXME: Better error needed here.
                    Syntax::Expr(expr) => {
                        return Unwind::error_at(expr.source_location(), "Expression at toplevel")
                    }
                };
            }
            Ok(())
        })?;
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
    /// Creates a new environment toplevel environment
    fn toplevel_env(&self) -> Env {
        self.foo.toplevel_env()
    }

    /// Creates a new environment enclosed by this one, containing one additional
    /// binding.
    pub fn bind(&self, name: &str, binding: Binding) -> Env {
        let child = self.enclose();
        child.ensure_binding(name, binding);
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

    pub fn ensure_binding(&self, name: &str, binding: Binding) {
        // println!("add binding: {}", name);
        self.env_ref.ensure_binding(name, binding);
    }

    pub fn set(&self, name: &str, value: Object) -> Option<Eval> {
        self.env_ref.set(name, value, self)
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
            Panic(panic) => self.eval_panic(panic),
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
        Ok(self.foo.into_array(data, None))
    }

    fn eval_bind(&self, bind: &Bind) -> Eval {
        let value = self.eval(&bind.value)?;
        let binding = match bind.typename {
            None => Binding::untyped(value),
            Some(ref typename) => {
                let typed = self.find_type(typename).source(&bind.source_location)?;
                match Binding::typed(typed, value, self) {
                    Ok(ok) => ok,
                    Err(mut unwind) => {
                        unwind.add_source_location(&bind.value.source_location());
                        return Err(unwind);
                    }
                }
            }
        };
        let tmp = binding.value.clone();
        // FIXME: the toplevel environment be marked as workspace to allow this,
        // or even better this should arrange to return the new environment somehow,
        // so that successive lets of same names each would create a new binding
        // and environment.
        let env = if bind.dynamic {
            self.clone()
        } else if self.is_toplevel() {
            self.ensure_binding(&bind.name, binding);
            self.clone()
        } else {
            self.bind(&bind.name, binding)
        };
        match &bind.body {
            None => Ok(tmp),
            Some(body) => {
                if bind.dynamic {
                    let old = if let Some(old) = env.get(&bind.name) {
                        env.set(&bind.name, tmp);
                        old
                    } else {
                        return Unwind::error_at(
                            bind.source_location.clone(),
                            &format!("Dynamic variable has no definition: {}", &bind.name),
                        );
                    };
                    let res = env.eval(&body);
                    env.set(&bind.name, old);
                    res
                } else {
                    env.eval(&body)
                }
            }
        }
    }

    fn eval_block(&self, block: &Block) -> Eval {
        let mut args = vec![];
        let mut parameter_types = vec![];
        for p in &block.params {
            args.push(Arg::new(p.source_location.clone(), p.name.clone()));
            parameter_types.push(&p.typename);
        }
        self.foo.make_closure(&self, args, (*block.body).clone(), parameter_types, &block.rtype)
    }

    fn eval_cascade(&self, cascade: &Cascade) -> Eval {
        let receiver = self.eval(&cascade.receiver)?;
        let mut res = receiver.clone();
        for messages in &cascade.chains {
            res = self.eval_sends(receiver.clone(), messages)?;
        }
        Ok(res)
    }

    fn check_not_defined(
        &self,
        name: &str,
        source_location: &SourceLocation,
    ) -> Result<(), Unwind> {
        if self.has_definition(name) {
            return Unwind::error_at(source_location.clone(), &format!("Cannot redefine {}", name));
        };
        Ok(())
    }

    pub fn load_module<P: AsRef<Path>>(&self, path: P) -> Result<Env, Unwind> {
        let mut file = path.as_ref().to_path_buf();
        if file.is_relative() {
            let name = match path.as_ref().components().next() {
                Some(std::path::Component::Normal(p)) => AsRef::<Path>::as_ref(p).to_str().unwrap(),
                _ => panic!("Bad module path! {}", path.as_ref().display()),
            };
            file = match self.foo.roots.get(name) {
                Some(p) => p.join(path),
                None => {
                    return Unwind::error(&format!(
                        "Unknown module: {}, --use /path/to/{} missing from command-line?",
                        name, name
                    ))
                }
            };
        }
        {
            // For some reason on 1.40 at least borrow() fails to infer type.
            let modules = self.foo.modules.borrow_mut();
            if let Some(module) = modules.get(&file) {
                return Ok(module.clone());
            }
        }
        let env = self.foo.load_module_into(&file, self.toplevel_env())?;
        self.foo.modules.borrow_mut().insert(file.clone(), env.clone());
        Ok(env)
    }

    fn do_class(&self, definition: &ClassDef) -> Eval {
        // println!("CLASS env: {:?}", self);
        let name = &definition.name;
        self.check_not_defined(name, &definition.source_location)?;
        let class = self.foo.make_class(definition, self).source(&definition.source_location)?;
        self.define(name, class.clone());
        Ok(class)
    }

    fn do_define(&self, definition: &DefineDef) -> Eval {
        let name = &definition.name;
        self.check_not_defined(name, &definition.source_location)?;
        let value = self.enclose().eval(&definition.init)?;
        self.define(name, value.clone());
        Ok(value)
    }

    fn do_extension(&self, extension: &ExtensionDef) -> Eval {
        let class = self.find_global_or_unwind(&extension.name)?;
        class.extend_class(extension, self)
    }

    fn do_import(&self, import: &ImportDef) -> Eval {
        assert!(self.is_toplevel());
        let n = self.env_ref.frame.borrow().len();
        let module = &self.load_module(&import.path)?.env_ref;
        assert_eq!(n, self.env_ref.frame.borrow().len());
        let res = match &import.name {
            None => self.env_ref.import_prefixed(&module, &import.prefix),
            Some(name) if name == "*" => self.env_ref.import_everything(&module),
            Some(name) => self.env_ref.import_name(&module, name),
        };
        if let Err(mut unwind) = res {
            unwind.add_source_location(&import.source_location);
            return Err(unwind);
        }
        Ok(self.foo.make_string(&import.path.to_string_lossy()))
    }

    fn do_interface(&self, interface: &InterfaceDef) -> Eval {
        let name = &interface.name;
        self.check_not_defined(&interface.name, &interface.source_location)?;
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

    pub fn maybe_type(&self, maybe_name: &Option<String>) -> Result<Option<Object>, Unwind> {
        match maybe_name {
            None => return Ok(None),
            Some(name) => Ok(Some(self.find_type(name)?)),
        }
    }

    // FIXME: should verify that the object is a type
    //
    // This MUST use get() instead of find_global because methods are in a
    // special environment which includes the class as well, even though the
    // definition is not yet complete.
    pub fn find_type(&self, name: &str) -> Eval {
        match self.get(name) {
            None => Unwind::error(&format!("Undefined type: '{}'", name)),
            Some(obj) => Ok(obj.clone()),
        }
    }

    pub fn find_interface(&self, name: &str) -> Eval {
        match self.get(name) {
            None => Unwind::error(&format!("Undefined interface: {}", name)),
            Some(obj) => match &obj.datum {
                Datum::Class(ref class) if class.interface => Ok(obj.clone()),
                _ => Unwind::error(&format!("Not an interface: {}", name)),
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
            Literal::Selector(value) => Ok(self.find_global_or_unwind("Selector")?.send(
                "intern:",
                &[self.foo.make_string(value)],
                self,
            )?),
        }
    }

    pub fn import_everything(&self, module: &Env) -> Eval {
        self.env_ref.import_everything(&module.env_ref)?;
        Ok(self.foo.make_boolean(true))
    }

    fn eval_panic(&self, panic: &Panic) -> Eval {
        Unwind::error_at(panic.source_location.clone(), self.eval(&panic.value)?.string_as_str())
    }

    fn eval_return(&self, ret: &Return) -> Eval {
        match self.home() {
            None => Unwind::error_at(ret.source_location.clone(), "Nothing to return from"),
            Some(env) => Unwind::return_from(env, self.eval(&ret.value)?),
        }
    }

    fn eval_sends(&self, mut receiver: Object, messages: &Vec<Message>) -> Eval {
        /*
        let orig_receiver = receiver.clone();
        unsafe {
            println!("{} CHAIN TO: {:?}", &DEBUG_DEPTH, &receiver);
            DEBUG_DEPTH += 1;
        } */
        for message in messages {
            // unsafe { println!("{} <- #{}", &DEBUG_DEPTH, message.selector.as_str()); }
            let mut values = Vec::new();
            values.reserve(message.args.len());
            for arg in &message.args {
                values.push(self.eval(arg)?);
            }
            receiver = receiver
                .send(message.selector.as_str(), &values[..], &self)
                .source(&message.source_location)?;
            // unsafe { println!("{} -> #{} ok!", &DEBUG_DEPTH, message.selector.as_str()); }
        }
        /*
        unsafe {
            DEBUG_DEPTH -= 1;
            println!("{} CHAIN OK: {:?}", &DEBUG_DEPTH, orig_receiver);
        }
        */
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
        let value = self.eval(&typecheck.expr)?;
        let typed = self.find_type(&typecheck.typename).source(&typecheck.source_location)?;
        typed.send("typecheck:", &[value], self).source_expr(&typecheck.expr)
    }

    fn eval_assign(&self, assign: &Assign) -> Eval {
        let value = self.eval(&assign.value)?;
        match self.set(&assign.name, value.clone()) {
            Some(res) => res.source_expr(&assign.value),
            None => {
                if let Some(receiver) = self.receiver() {
                    if let Some(slot) = receiver.slots().iter().find(|s| &s.name == &assign.name) {
                        return write_instance_variable(&receiver, slot, value, self)
                            .source_expr(&assign.value);
                    }
                }
                // FIXME: there used to be a workspace lookup here...
                Unwind::error_at(
                    assign.source_location.clone(),
                    &format!("Cannot assign to an unbound variable: {}", &assign.name),
                )
            }
        }
    }

    fn eval_var(&self, var: &Var) -> Eval {
        match self.get(&var.name) {
            Some(value) => return Ok(value),
            None => {
                if let Some(receiver) = self.receiver() {
                    if let Some(slot) = receiver.slots().iter().find(|s| &s.name == &var.name) {
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
        Unwind::error_at(var.source_location.clone(), &format!("Unbound variable: {}", &var.name))
    }
}

#[cfg(test)]
pub mod utils {

    use crate::eval::*;

    pub fn eval_exception(source: &str) -> (Unwind, Env) {
        let env = Env::new();
        match env.eval_all(source) {
            Err(unwind) => match &unwind {
                Unwind::Panic(..) => (unwind, env),
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
            Err(Unwind::Panic(error, location)) => {
                panic!("Panic in eval_ok: {}:\n{}", error.what(), location.context());
            }
            Err(Unwind::ReturnFrom(..)) => panic!("Unexpected return-from in eval_ok"),
        }
    }
}
