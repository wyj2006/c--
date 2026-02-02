use crate::{
    ast::{
        Attribute,
        decl::{FunctionSpec, StorageClass},
    },
    ctype::{RecordKind, Type, TypeKind, is_compatible},
    diagnostic::{Diagnostic, DiagnosticKind},
};
use indexmap::IndexMap;
use num::BigInt;
use pest::Span;
use std::{cell::RefCell, collections::HashMap, iter::repeat, ptr::addr_of, rc::Rc};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Namespace {
    Label,
    Tag,
    Member,
    Ordinary,
}

#[derive(Debug)]
pub struct SymbolTable<'a> {
    pub namespaces: HashMap<Namespace, IndexMap<String, Rc<RefCell<Symbol<'a>>>>>,
    pub parent: Option<Rc<RefCell<SymbolTable<'a>>>>,
    pub children: Vec<Rc<RefCell<SymbolTable<'a>>>>,
}

#[derive(Debug, Clone)]
pub struct Symbol<'a> {
    pub define_span: Option<Span<'a>>,
    pub declare_spans: Vec<Span<'a>>,
    pub name: String,
    pub kind: SymbolKind<'a>,
    pub r#type: Rc<RefCell<Type<'a>>>,
    pub attributes: Vec<Rc<RefCell<Attribute<'a>>>>,
}

#[derive(Debug, Clone)]
pub enum SymbolKind<'a> {
    Label,
    Record {
        kind: RecordKind,
    },
    Enum,
    Object {
        storage_classes: Vec<StorageClass<'a>>,
    },
    Member {
        bit_field: Option<BigInt>,
    },
    Function {
        function_specs: Vec<FunctionSpec<'a>>,
    },
    Parameter {
        storage_classes: Vec<StorageClass<'a>>,
    },
    EnumConst {
        value: BigInt,
    },
    Type,
}

impl<'a> SymbolTable<'a> {
    pub fn new() -> SymbolTable<'a> {
        SymbolTable {
            namespaces: HashMap::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn print(&self, indent: usize) {
        println!(
            "{}{:p}",
            repeat("  ").take(indent).collect::<String>(),
            addr_of!(*self)
        );
        for (namespace, symbols) in &self.namespaces {
            println!(
                "{}{namespace:?}",
                repeat("  ").take(indent + 1).collect::<String>(),
            );
            for (name, symbol) in symbols {
                let symbol = symbol.borrow();
                println!(
                    "{}{}  {}({:p})  {}  {}",
                    repeat("  ").take(indent + 2).collect::<String>(),
                    name,
                    symbol.r#type.borrow().to_string(),
                    symbol.r#type.as_ptr(),
                    match &symbol.kind {
                        SymbolKind::Object { storage_classes }
                        | SymbolKind::Parameter { storage_classes } => storage_classes
                            .iter()
                            .map(|x| x.kind.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                        SymbolKind::Function { function_specs } => function_specs
                            .iter()
                            .map(|x| x.kind.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                        _ => format!(""),
                    },
                    format!("{:?}", symbol.attributes)
                );
                match &symbol.r#type.borrow().kind {
                    TypeKind::Record { members, .. } => {
                        if let Some(t) = members {
                            for (name, member) in t {
                                let SymbolKind::Member { bit_field } = &member.borrow().kind else {
                                    unreachable!();
                                };
                                println!(
                                    "{}{name}{}  {}({:p})",
                                    repeat("  ").take(indent + 3).collect::<String>(),
                                    match bit_field {
                                        Some(t) => format!(":{t}"),
                                        None => String::new(),
                                    },
                                    member.borrow().r#type.borrow().to_string(),
                                    member.borrow().r#type.as_ptr()
                                );
                            }
                        }
                    }
                    TypeKind::Enum { enum_consts, .. } => {
                        if let Some(t) = enum_consts {
                            for (name, enum_const) in t {
                                let SymbolKind::EnumConst { value } = &enum_const.borrow().kind
                                else {
                                    unreachable!();
                                };
                                println!(
                                    "{}{name}:  {value:?}",
                                    repeat("  ").take(indent + 3).collect::<String>(),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        for child in &self.children {
            child.borrow().print(indent + 1);
        }
    }

    pub fn add(
        &mut self,
        namespace: Namespace,
        symbol: Rc<RefCell<Symbol<'a>>>,
    ) -> Result<(), Diagnostic<'a>> {
        let name = symbol.borrow().name.clone();
        if name.len() == 0 {
            return Ok(());
        }
        if let Some(symbols) = self.namespaces.get_mut(&namespace) {
            match symbols.get(&name) {
                Some(pre_symbol) => {
                    let mut pre_symbol = pre_symbol.borrow_mut();
                    let symbol = symbol.borrow();
                    if !is_compatible(Rc::clone(&pre_symbol.r#type), Rc::clone(&symbol.r#type)) {
                        Err(Diagnostic {
                            span: symbol.define_span.unwrap_or(symbol.declare_spans[0]),
                            kind: DiagnosticKind::Error,
                            message: format!(
                                "redefine '{name}' with incompatible type '{}' and '{}'",
                                symbol.r#type.borrow().to_string(),
                                pre_symbol.r#type.borrow().to_string()
                            ),
                            notes: vec![Diagnostic {
                                span: pre_symbol
                                    .define_span
                                    .unwrap_or(pre_symbol.declare_spans[0]),
                                kind: DiagnosticKind::Note,
                                message: format!("previous defination"),
                                notes: vec![],
                            }],
                        })
                    } else if let Some(a) = pre_symbol.define_span
                        && let Some(b) = symbol.define_span
                    {
                        Err(Diagnostic {
                            span: b,
                            kind: DiagnosticKind::Error,
                            message: format!("'{name}' is redefined"),
                            notes: vec![Diagnostic {
                                span: a,
                                kind: DiagnosticKind::Note,
                                message: format!("previous defination"),
                                notes: vec![],
                            }],
                        })
                    } else {
                        pre_symbol.define_span = if let Some(t) = symbol.define_span {
                            Some(t)
                        } else {
                            pre_symbol.define_span
                        };
                        pre_symbol
                            .declare_spans
                            .extend(symbol.declare_spans.clone());
                        Ok(())
                    }
                }
                None => {
                    symbols.insert(name, symbol);
                    Ok(())
                }
            }
        } else {
            let mut map = IndexMap::new();
            map.insert(name, symbol);
            self.namespaces.insert(namespace, map);
            Ok(())
        }
    }

    ///只查找当前作用域
    pub fn lookup_current<'b>(
        &self,
        namespace: Namespace,
        name: &'b String,
    ) -> Option<Rc<RefCell<Symbol<'a>>>> {
        if let Some(symbols) = self.namespaces.get(&namespace) {
            match symbols.get(name) {
                Some(t) => Some(Rc::clone(t)),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn lookup<'b>(
        &self,
        namespace: Namespace,
        name: &'b String,
    ) -> Option<Rc<RefCell<Symbol<'a>>>> {
        match self.lookup_current(namespace, name) {
            Some(t) => Some(t),
            None => match &self.parent {
                Some(t) => t.borrow().lookup(namespace, name),
                None => None,
            },
        }
    }
}
