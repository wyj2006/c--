use std::{
    cell::RefCell, collections::HashMap, iter::repeat, mem::discriminant, ptr::addr_of, rc::Rc,
};

use indexmap::IndexMap;
use num::BigInt;
use pest::Span;

use crate::{
    ast::{
        Attribute,
        decl::{FunctionSpec, StorageClass},
    },
    ctype::{Type, TypeKind, is_compatible},
    diagnostic::{Error, ErrorKind},
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Namespace {
    Label,
    Tag,
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
    Record,
    Enum,
    Object {
        storage_classes: Vec<StorageClass<'a>>,
    },
    Member {
        bit_field: Option<usize>,
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
            addr_of!(self)
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
                        | SymbolKind::Parameter { storage_classes } =>
                            format!("{storage_classes:?}"),
                        SymbolKind::Function { function_specs } => format!("{function_specs:?}"),
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
    ) -> Result<(), Error<'a>> {
        let name = symbol.borrow().name.clone();
        if let Some(symbols) = self.namespaces.get_mut(&namespace) {
            match symbols.get(&name) {
                Some(pre_symbol) => {
                    let mut pre_symbol = pre_symbol.borrow_mut();
                    let symbol = symbol.borrow();
                    if let Some(_) = pre_symbol.define_span
                        && let Some(_) = symbol.define_span
                    {
                        Err(Error {
                            span: symbol.define_span.unwrap_or(symbol.declare_spans[0]),
                            kind: ErrorKind::Redefine(name),
                        })
                    } else if discriminant(&pre_symbol.kind) != discriminant(&symbol.kind) {
                        Err(Error {
                            span: symbol.define_span.unwrap_or(symbol.declare_spans[0]),
                            kind: ErrorKind::Redefine(name),
                        })
                    } else if !is_compatible(
                        Rc::clone(&pre_symbol.r#type),
                        Rc::clone(&symbol.r#type),
                    ) {
                        Err(Error {
                            span: symbol.define_span.unwrap_or(symbol.declare_spans[0]),
                            kind: ErrorKind::Redefine(name),
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

    pub fn lookup<'b>(
        &self,
        namespace: Namespace,
        name: &'b String,
    ) -> Option<Rc<RefCell<Symbol<'a>>>> {
        if let Some(symbols) = self.namespaces.get(&namespace) {
            match symbols.get(name) {
                Some(t) => return Some(Rc::clone(t)),
                None => {}
            }
        }
        if let Some(t) = &self.parent {
            t.borrow().lookup(namespace, name)
        } else {
            None
        }
    }
}
