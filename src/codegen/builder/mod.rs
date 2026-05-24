pub mod llvmir;

use crate::{
    ast::{
        decl::{FunctionSpec, StorageClass},
        expr::{BinOpKind, CastMethod, UnaryOpKind},
    },
    ctype::{
        Type,
        layout::{ConstDesignation, Layout},
    },
    symtab::{Namespace, Symbol, SymbolTable},
    variant::Variant,
};
use codespan::Span;
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait Builder {
    type Value: Clone;
    type BasicBlock: Clone;

    fn append_context(&mut self, file_id: usize, span: Span);
    fn pop_context(&mut self);

    fn enter_scope(&mut self, symtab: &Rc<RefCell<SymbolTable>>);
    fn leave_scope(&mut self);
    fn lookup(&self, name: &str, namespace: Namespace) -> Option<Rc<RefCell<Symbol>>>;

    fn enter_function(&mut self, function: &Self::Value);
    fn leave_function(&mut self, function: &Self::Value) -> Result<(), Diagnostic<usize>>;

    //在当前基本块后面加入基本块
    fn append_basic_block(&mut self, name: &str) -> Result<Self::BasicBlock, Diagnostic<usize>>;
    fn current_basic_block(&self) -> Result<Self::BasicBlock, Diagnostic<usize>>;
    fn position_at_end(&self, basic_block: &Self::BasicBlock);
    fn get_previous_block(&self, basic_block: &Self::BasicBlock) -> Option<Self::BasicBlock>;

    //variant和type的类型不一定一样, 也就是说不会对variant进行隐式转换
    fn variant_to_value(
        &self,
        variant: &Variant,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn layout_to_value(
        &self,
        layout: Layout,
        path: Vec<ConstDesignation>,
        init_values: &HashMap<Vec<ConstDesignation>, Self::Value>,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn decl_variable(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[StorageClass],
        init_value: Option<Self::Value>,
        vla_size: Option<Self::Value>, //vla的长度
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn decl_function(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[StorageClass],
        function_specs: &[FunctionSpec],
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn decl_parameter(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn decl_record(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(), Diagnostic<usize>>;

    fn conditional_branch(
        &mut self,
        condition: &Self::Value,
        then_block: &Self::BasicBlock,
        else_block: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>>;

    fn unconditional_branch(
        &mut self,
        dest_block: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>>;

    fn switch(
        &mut self,
        condition: &Self::Value,
        cases: &[(Self::Value, Self::BasicBlock)],
        default: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>>;

    fn r#return(&mut self, value: Option<Self::Value>) -> Result<(), Diagnostic<usize>>;

    fn load(
        &mut self,
        ptr: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn load_var(&mut self, name: &str) -> Result<Self::Value, Diagnostic<usize>>;

    fn load_member(
        &mut self,
        target_value: &Self::Value,
        record_type: &Rc<RefCell<Type>>,
        member_name: &str,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn load_compound_literal(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[StorageClass],
        init_value: &Self::Value,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn load_bitfield(
        &mut self,
        value: &Self::Value,
        width: usize,
        offset: usize,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn store(&mut self, ptr: &Self::Value, value: &Self::Value) -> Result<(), Diagnostic<usize>>;

    fn store_bitfield(
        &mut self,
        ptr: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
        value: &Self::Value,
        width: usize,
        offset: usize,
    ) -> Result<(), Diagnostic<usize>>;

    fn cast(
        &mut self,
        target_value: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
        method: CastMethod,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn subscript(
        &mut self,
        target: &Self::Value,
        index: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn call(
        &mut self,
        function: &Self::Value,
        args: &[Self::Value],
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn phi(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        incomings: &[(Self::Value, Self::BasicBlock)],
    ) -> Result<Self::Value, Diagnostic<usize>>;

    fn unaryop(
        &mut self,
        op: UnaryOpKind,
        operand_value: &Self::Value,
        operand_type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>>;

    //op不包括像And, Or这样包含控制流的运算
    fn binop(
        &mut self,
        op: BinOpKind,
        left_value: &Self::Value,
        left_type: &Rc<RefCell<Type>>,
        right_value: &Self::Value,
        right_type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>>;
}
