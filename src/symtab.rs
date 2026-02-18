use crate::{
    ast::{
        Attribute,
        decl::{FunctionSpec, StorageClass},
    },
    ctype::{RecordKind, Type, TypeKind, is_compatible},
};
use codespan::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use indexmap::IndexMap;
use num::BigInt;
use std::{cell::RefCell, collections::HashMap, iter::repeat, ptr::addr_of, rc::Rc};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Namespace {
    Label,
    Tag,
    Member,
    Ordinary,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub namespaces: HashMap<Namespace, IndexMap<String, Rc<RefCell<Symbol>>>>,
    pub parent: Option<Rc<RefCell<SymbolTable>>>,
    pub children: Vec<Rc<RefCell<SymbolTable>>>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub define_loc: Option<(usize, Span)>,
    pub declare_locs: Vec<(usize, Span)>,
    pub name: String,
    pub kind: SymbolKind,
    pub r#type: Rc<RefCell<Type>>,
    pub attributes: Vec<Rc<RefCell<Attribute>>>,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Label,
    Record {
        kind: RecordKind,
    },
    Enum,
    Object {
        storage_classes: Vec<StorageClass>,
    },
    Member {
        bit_field: Option<usize>,
        index: usize,
    },
    Function {
        function_specs: Vec<FunctionSpec>,
    },
    Parameter {
        storage_classes: Vec<StorageClass>,
        //参数位置, 从0开始计
        index: u32,
    },
    EnumConst {
        value: BigInt,
    },
    Type,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
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
                    "{}{}  {}({:p})  {}  [[{}]]",
                    repeat("  ").take(indent + 2).collect::<String>(),
                    name,
                    symbol.r#type.borrow().to_string(),
                    symbol.r#type.as_ptr(),
                    match &symbol.kind {
                        SymbolKind::Object { storage_classes }
                        | SymbolKind::Parameter {
                            storage_classes, ..
                        } => storage_classes
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
                    symbol
                        .attributes
                        .iter()
                        .map(|x| {
                            let prefix_name = &x.borrow().prefix_name;
                            let name = &x.borrow().name;
                            if let Some(t) = prefix_name {
                                format!("{t}::{name}")
                            } else {
                                format!("{name}")
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                match &symbol.r#type.borrow().kind {
                    TypeKind::Record { members, .. } => {
                        if let Some(t) = members {
                            for (name, member) in t {
                                let SymbolKind::Member { bit_field, .. } = &member.borrow().kind
                                else {
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
        symbol: Rc<RefCell<Symbol>>,
    ) -> Result<(), Diagnostic<usize>> {
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
                        let symbol_loc = symbol.define_loc.unwrap_or(symbol.declare_locs[0]);
                        let pre_symbol_loc =
                            pre_symbol.define_loc.unwrap_or(pre_symbol.declare_locs[0]);
                        Err(Diagnostic::error()
                            .with_message(format!(
                                "redefine '{name}' with incompatible type '{}' and '{}'",
                                symbol.r#type.borrow().to_string(),
                                pre_symbol.r#type.borrow().to_string()
                            ))
                            .with_label(
                                Label::primary(symbol_loc.0, symbol_loc.1)
                                    .with_message("current defination"),
                            )
                            .with_label(
                                Label::secondary(pre_symbol_loc.0, pre_symbol_loc.1)
                                    .with_message("previous defination"),
                            ))
                    } else if let Some(a) = pre_symbol.define_loc
                        && let Some(b) = symbol.define_loc
                    {
                        Err(Diagnostic::error()
                            .with_message(format!("'{name}' is redefined"))
                            .with_label(Label::primary(b.0, b.1).with_message("current defination"))
                            .with_label(
                                Label::secondary(a.0, a.1).with_message("previous defination"),
                            ))
                    } else {
                        pre_symbol.define_loc = if let Some(t) = symbol.define_loc {
                            Some(t)
                        } else {
                            pre_symbol.define_loc
                        };
                        pre_symbol.declare_locs.extend(symbol.declare_locs.clone());
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
    ) -> Option<Rc<RefCell<Symbol>>> {
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
    ) -> Option<Rc<RefCell<Symbol>>> {
        match self.lookup_current(namespace, name) {
            Some(t) => Some(t),
            None => match &self.parent {
                Some(t) => t.borrow().lookup(namespace, name),
                None => None,
            },
        }
    }
}
