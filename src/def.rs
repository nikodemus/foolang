use std::path::{Path, PathBuf};

use crate::expr::*;
use crate::source_location::{SourceLocation, Span, TweakSpan};
use crate::unwind::Unwind;

#[derive(Debug, PartialEq)]
pub enum Def {
    ClassDef(ClassDef),
    DefineDef(DefineDef),
    ExtensionDef(ExtensionDef),
    ImportDef(ImportDef),
    InterfaceDef(InterfaceDef),
}

impl Def {
    #[cfg(test)]
    pub fn add_method(&mut self, kind: MethodKind, method: MethodDefinition) {
        match self {
            Def::ClassDef(class) => class.add_method(kind, method),
            _ => panic!("BUG: trying to add a method to {:?}", self),
        }
    }

    pub fn span(&self) -> Span {
        use Def::*;
        match self {
            ClassDef(definition) => return definition.source_location.get_span(),
            DefineDef(definition) => return definition.source_location.get_span(),
            ExtensionDef(extension) => return extension.source_location.get_span(),
            ImportDef(import) => return import.source_location.get_span(),
            InterfaceDef(interface) => return interface.source_location.get_span(),
        };
    }

    pub fn shift_span(&mut self, n: usize) {
        self.tweak_span(n, 0);
    }

    pub fn extend_span(&mut self, n: isize) {
        self.tweak_span(0, n);
    }

    fn tweak_span(&mut self, shift: usize, extend: isize) {
        match self {
            Def::ClassDef(class) => class.tweak_span(shift, extend),
            Def::DefineDef(def) => def.tweak_span(shift, extend),
            Def::ExtensionDef(ext) => {
                ext.source_location.tweak(shift, extend);
                for m in &mut ext.instance_methods {
                    m.tweak_span(shift, extend);
                }
                for m in &mut ext.class_methods {
                    m.tweak_span(shift, extend);
                }
            }
            Def::ImportDef(import) => {
                import.source_location.tweak(shift, extend);
            }
            Def::InterfaceDef(interface) => interface.tweak_span(shift, extend),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassDef {
    pub source_location: SourceLocation,
    pub name: String,
    pub instance_variables: Vec<Var>,
    pub instance_methods: Vec<MethodDefinition>,
    pub class_methods: Vec<MethodDefinition>,
    pub interfaces: Vec<String>,
    pub default_constructor: Option<String>,
}

impl ClassDef {
    pub fn new(
        source_location: SourceLocation,
        name: String,
        instance_variables: Vec<Var>,
    ) -> ClassDef {
        ClassDef {
            source_location,
            name,
            instance_variables,
            instance_methods: Vec::new(),
            class_methods: Vec::new(),
            interfaces: Vec::new(),
            default_constructor: None,
        }
    }

    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.source_location.tweak(shift, extend);
        for var in &mut self.instance_variables {
            var.source_location.tweak(shift, extend);
        }
        for m in &mut self.instance_methods {
            m.tweak_span(shift, extend);
        }
        for m in &mut self.class_methods {
            m.tweak_span(shift, extend);
        }
    }

    #[cfg(test)]
    pub fn syntax(span: Span, name: String, instance_variables: Vec<Var>) -> Def {
        Def::ClassDef(ClassDef::new(SourceLocation::span(&span), name, instance_variables))
    }

    pub fn add_interface(&mut self, name: &str) {
        self.interfaces.push(name.to_string())
    }

    pub fn add_method(&mut self, kind: MethodKind, method: MethodDefinition) {
        match kind {
            MethodKind::Instance => self.instance_methods.push(method),
            MethodKind::Class => self.class_methods.push(method),
            _ => panic!("Cannot add {:?} to a ClassDef", kind),
        };
    }

    pub fn constructor(&self) -> String {
        if self.instance_variables.is_empty() {
            match &self.default_constructor {
                Some(ctor) => ctor.to_string(),
                None => "new".to_string(),
            }
        } else {
            let mut selector = String::new();
            for var in &self.instance_variables {
                selector.push_str(&var.name);
                selector.push_str(":");
            }
            selector
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DefineDef {
    pub source_location: SourceLocation,
    pub name: String,
    pub init: Expr,
}

impl DefineDef {
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.source_location.tweak(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExtensionDef {
    pub source_location: SourceLocation,
    pub name: String,
    pub instance_methods: Vec<MethodDefinition>,
    pub class_methods: Vec<MethodDefinition>,
    pub interfaces: Vec<String>,
}

impl ExtensionDef {
    pub fn new(source_location: SourceLocation, name: &str) -> Self {
        Self {
            source_location,
            name: name.to_string(),
            instance_methods: Vec::new(),
            class_methods: Vec::new(),
            interfaces: Vec::new(),
        }
    }

    pub fn add_interface(&mut self, name: &str) {
        self.interfaces.push(name.to_string())
    }

    pub fn add_method(&mut self, kind: MethodKind, method: MethodDefinition) {
        match kind {
            MethodKind::Instance => self.instance_methods.push(method),
            MethodKind::Class => self.class_methods.push(method),
            _ => panic!("Cannot add {:?} to a ExtensionDef", kind),
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportDef {
    pub source_location: SourceLocation,
    pub path: PathBuf,
    pub prefix: String,
    pub name: Option<String>,
}

impl ImportDef {
    pub fn def<P: AsRef<Path>>(
        source_location: SourceLocation,
        path: P,
        prefix: &str,
        name: Option<&str>,
    ) -> Def {
        Def::ImportDef(ImportDef {
            source_location,
            path: path.as_ref().to_path_buf(),
            prefix: prefix.to_string(),
            name: name.map(|x| x.to_string()),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InterfaceDef {
    pub source_location: SourceLocation,
    pub name: String,
    pub instance_methods: Vec<MethodDefinition>,
    pub class_methods: Vec<MethodDefinition>,
    pub required_methods: Vec<MethodDefinition>,
    pub interfaces: Vec<String>,
}

impl InterfaceDef {
    pub fn new(source_location: SourceLocation, name: &str) -> InterfaceDef {
        InterfaceDef {
            source_location,
            name: name.to_string(),
            instance_methods: Vec::new(),
            class_methods: Vec::new(),
            required_methods: Vec::new(),
            interfaces: Vec::new(),
        }
    }

    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.source_location.tweak(shift, extend);
        for m in &mut self.instance_methods {
            m.tweak_span(shift, extend);
        }
        for m in &mut self.class_methods {
            m.tweak_span(shift, extend);
        }
    }

    pub fn add_method(&mut self, kind: MethodKind, method: MethodDefinition) {
        match kind {
            MethodKind::Instance => self.instance_methods.push(method),
            MethodKind::Class => self.class_methods.push(method),
            MethodKind::Required => self.required_methods.push(method),
        };
    }

    pub fn add_interface(&mut self, name: &str) {
        self.interfaces.push(name.to_string())
    }
}

// FIXME: split into signature and method
#[derive(Debug, PartialEq, Clone)]
pub struct MethodDefinition {
    pub source_location: SourceLocation,
    pub selector: String,
    pub parameters: Vec<Var>,
    pub body: Option<Box<Expr>>,
    pub return_type: Option<String>,
}

impl MethodDefinition {
    pub fn new(
        source_location: SourceLocation,
        selector: String,
        parameters: Vec<Var>,
        return_type: Option<String>,
    ) -> MethodDefinition {
        MethodDefinition {
            source_location,
            selector,
            parameters,
            body: None,
            return_type,
        }
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.source_location.tweak(shift, extend);
        for var in &mut self.parameters {
            var.source_location.tweak(shift, extend);
        }
        match &mut self.body {
            Some(ref mut span) => span.tweak_span(shift, extend),
            _ => (),
        }
    }
    pub fn required_body(&self) -> Result<&Expr, Unwind> {
        match &self.body {
            Some(body) => Ok(&(*body)),
            None => {
                return Unwind::error_at(
                    self.source_location.clone(),
                    "Partial methods not allowed here",
                );
            }
        }
    }
}

#[derive(Debug)]
pub enum MethodKind {
    Class,
    Instance,
    Required,
}
