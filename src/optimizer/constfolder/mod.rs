pub mod fold_decl;
pub mod fold_expr;
pub mod fold_init;
pub mod fold_stmt;
#[cfg(test)]
pub mod tests;

use crate::{ast::TranslationUnit, symtab::SymbolTable, variant::Variant};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

///同时完成常量折叠和常量传播优化
pub struct ConstFolder {
    //break语句的OUT
    //第一个键对应的是break跳出的循环语句, 第二个键对应的是该break语句
    pub break_outs: HashMap<usize, HashMap<usize, HashMap<String, Variant>>>,
    //continue语句的OUT
    pub continue_outs: HashMap<usize, HashMap<usize, HashMap<String, Variant>>>,
    //case语句(包括default)的IN
    pub case_ins: HashMap<usize, HashMap<usize, HashMap<String, Variant>>>,
    //label的IN
    pub label_ins: HashMap<usize, HashMap<usize, HashMap<String, Variant>>>,
    pub func_symtabs: Vec<Rc<RefCell<SymbolTable>>>,
}

impl ConstFolder {
    pub fn new() -> ConstFolder {
        ConstFolder {
            break_outs: HashMap::new(),
            continue_outs: HashMap::new(),
            case_ins: HashMap::new(),
            label_ins: HashMap::new(),
            func_symtabs: vec![],
        }
    }

    pub fn fold(&mut self, ast: Rc<RefCell<TranslationUnit>>) -> Result<(), Diagnostic<usize>> {
        self.visit_translation_unit(ast)
    }

    pub fn intersection(
        &self,
        a: &HashMap<String, Variant>,
        b: &HashMap<String, Variant>,
    ) -> HashMap<String, Variant> {
        let mut result = HashMap::new();
        //忽略不可达的数据流
        if b.contains_key(&String::new()) {
            return a.clone();
        }
        if a.contains_key(&String::new()) {
            return b.clone();
        }
        for (name, value) in b {
            match a.get(name) {
                Some(t) => {
                    if let Variant::Bool(true) = t.eq(&value) {
                        result.insert(name.clone(), value.clone());
                    }
                }
                None => {}
            }
        }
        result
    }

    ///阻止数据流传播
    pub fn prevent(&self, a: &HashMap<String, Variant>) -> HashMap<String, Variant> {
        let mut result = HashMap::new();
        for (name, _) in a {
            result.insert(name.clone(), Variant::Unknown);
        }
        result
    }

    ///传递给不可达代码
    pub fn unreachable(&self, a: &HashMap<String, Variant>) -> HashMap<String, Variant> {
        let mut result = self.prevent(a);
        //用空字符串标记
        result.insert(String::new(), Variant::Unknown);
        result
    }

    pub fn visit_translation_unit(
        &mut self,
        node: Rc<RefCell<TranslationUnit>>,
    ) -> Result<(), Diagnostic<usize>> {
        for decl in &node.borrow().decls {
            self.break_outs = HashMap::new();
            self.continue_outs = HashMap::new();
            self.case_ins = HashMap::new();
            self.label_ins = HashMap::new();

            let mut a = vec![
                self.break_outs.clone(),
                self.continue_outs.clone(),
                self.case_ins.clone(),
                self.label_ins.clone(),
            ];

            loop {
                self.visit_declaration(Rc::clone(&decl), HashMap::new())?;

                let t = vec![
                    self.break_outs.clone(),
                    self.continue_outs.clone(),
                    self.case_ins.clone(),
                    self.label_ins.clone(),
                ];
                if a == t {
                    break;
                }
                a = t;
            }
        }
        Ok(())
    }
}
