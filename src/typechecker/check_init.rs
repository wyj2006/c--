use crate::ast::expr::{EncodePrefix, Expr, ExprKind};
use crate::ast::{Designation, DesignationKind};
use crate::ctype::cast::remove_qualifier;
use crate::ctype::{TypeKind, is_compatible};
use crate::diagnostic::warning;
use crate::typechecker::{Context, TypeChecker};
use crate::variant::Variant;
use crate::{
    ast::{Initializer, InitializerKind},
    ctype::Type,
};
use codespan::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use num::{BigInt, ToPrimitive};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
struct DesignationNode {
    pub index: usize, //该节点是父节点的第几个子节点
    pub r#type: Rc<RefCell<Type>>,
    pub designation: SimpleDesignation,
}

#[derive(Debug)]
enum SimpleDesignation {
    Subscript(usize),
    MemberAccess(String),
}

impl DesignationNode {
    //该节点的第index个子节点, 不存在返回None
    pub fn child(&self, index: usize) -> Result<Option<DesignationNode>, Diagnostic<usize>> {
        let r#type = self.r#type.borrow();
        match &r#type.kind {
            TypeKind::Array {
                element_type,
                len_expr: Some(len_expr),
                ..
            } => match &len_expr.borrow().value {
                Variant::Int(value) => {
                    if BigInt::from(index) < *value {
                        Ok(Some(DesignationNode {
                            index,
                            r#type: Rc::clone(element_type),
                            designation: SimpleDesignation::Subscript(index),
                        }))
                    } else {
                        Ok(None)
                    }
                }
                _ => Err(Diagnostic::error()
                    .with_message(format!("the size of array must has integer type"))
                    .with_label(Label::primary(r#type.file_id, r#type.span))),
            },
            TypeKind::Array {
                element_type,
                len_expr: None,
                ..
            } => Ok(Some(DesignationNode {
                index,
                r#type: Rc::clone(element_type),
                designation: SimpleDesignation::Subscript(index),
            })),
            TypeKind::Record {
                members: Some(members),
                ..
            } => {
                if let Some((name, member)) = members.get_index(index) {
                    Ok(Some(DesignationNode {
                        index,
                        r#type: Rc::clone(&member.borrow().r#type),
                        designation: SimpleDesignation::MemberAccess(name.clone()),
                    }))
                } else {
                    Ok(None)
                }
            }
            TypeKind::Record { members: None, .. } => Err(Diagnostic::error()
                .with_message(format!("incomplete type: '{}'", r#type.to_string()))
                .with_label(Label::primary(r#type.file_id, r#type.span))),
            _ => Ok(None),
        }
    }
}

impl SimpleDesignation {
    pub fn to_designation(&self, file_id: usize, span: Span) -> Designation {
        match self {
            SimpleDesignation::Subscript(index) => Designation::new(
                file_id,
                span,
                DesignationKind::Subscript(Rc::new(RefCell::new(Expr::new_const_int(
                    file_id,
                    span,
                    *index,
                    Rc::new(RefCell::new(Type {
                        kind: TypeKind::ULongLong,
                        ..Type::new(file_id, span)
                    })),
                )))),
            ),
            SimpleDesignation::MemberAccess(name) => {
                Designation::new(file_id, span, DesignationKind::MemberAccess(name.clone()))
            }
        }
    }
}

impl TypeChecker {
    fn next_designation(
        &self,
        mut i: usize,
        path: &mut Vec<DesignationNode>,
    ) -> Result<(), Diagnostic<usize>> {
        while i < path.len() - 1 {
            path.pop();
        }
        loop {
            if i > 0
                && let Some(parent) = path.get(i - 1)
                && let Some(child) = path.get(i)
            {
                if let Some(t) = parent.child(child.index + 1)? {
                    *path.get_mut(i).unwrap() = t;
                    break;
                } else {
                    path.remove(i);
                    i -= 1;
                }
            } else {
                break;
            }
        }
        while i > 0
            && let Some(t) = path.last()
            && let Some(t) = t.child(0)?
        {
            path.push(t);
        }
        Ok(())
    }

    fn new_designation_path(
        &self,
        r#type: Rc<RefCell<Type>>,
    ) -> Result<Vec<DesignationNode>, Diagnostic<usize>> {
        let mut path = vec![DesignationNode {
            index: 0,
            r#type: Rc::clone(&r#type),
            designation: SimpleDesignation::Subscript(0), //随便设置的
        }];

        while let Some(t) = path.last().unwrap().child(0)? {
            path.push(t);
        }
        Ok(path)
    }

    pub fn visit_initializer(
        &mut self,
        node: Rc<RefCell<Initializer>>,
        r#type: Rc<RefCell<Type>>, //这个初始化的类型
    ) -> Result<(), Diagnostic<usize>> {
        self.contexts
            .push(Context::Init(node.borrow().kind.clone()));

        let mut node = node.borrow_mut();
        node.r#type = Rc::clone(&r#type);

        let mut max_index: usize = 0; //用于补全数组类型

        match &node.kind {
            InitializerKind::Braced(initializers) => {
                let mut path = self.new_designation_path(Rc::clone(&r#type))?;

                for (i, initializer) in initializers.iter().enumerate() {
                    //标量或者union只能有一个初始化器
                    if i > 0 {
                        if r#type.borrow().is_union() {
                            warning(
                                format!("excess element in union initializer"),
                                initializer.borrow().file_id,
                                initializer.borrow().span,
                                vec![],
                            );
                            continue;
                        } else if path.len() <= 1 {
                            warning(
                                format!("excess element in scale initializer"),
                                initializer.borrow().file_id,
                                initializer.borrow().span,
                                vec![],
                            );
                            continue;
                        }
                    }

                    let file_id = initializer.borrow().file_id;
                    let span = initializer.borrow().span;
                    let mut keep_num = 2; //保留的节点数

                    if initializer.borrow().designation.len() > 0 {
                        //从头开始匹配
                        path = self.new_designation_path(Rc::clone(&r#type))?;
                        let designations = &initializer.borrow().designation;
                        let mut i = 0;
                        //尝试匹配
                        while i < designations.len() && i + 1 < path.len() {
                            let j = i + 1;
                            let a = path[j].designation.to_designation(file_id, span);
                            let b = &designations[i];
                            self.visit_designation(b)?;
                            match (&a.kind, &b.kind) {
                                (DesignationKind::Subscript(x), DesignationKind::Subscript(y)) => {
                                    if x.borrow().value != y.borrow().value {
                                        self.next_designation(j, &mut path)?;
                                    } else {
                                        i += 1;
                                    }
                                }
                                (
                                    DesignationKind::MemberAccess(x),
                                    DesignationKind::MemberAccess(y),
                                ) => {
                                    if *x != *y {
                                        self.next_designation(j, &mut path)?;
                                    } else {
                                        i += 1;
                                    }
                                }
                                _ => self.next_designation(j, &mut path)?,
                            }
                        }
                        if i < designations.len() {
                            let b = &designations[i];
                            return match &b.kind {
                                DesignationKind::Subscript(index) => Err(Diagnostic::error()
                                    .with_message(format!(
                                        "array designator index ({}) exceeds array bounds",
                                        index.borrow().unparse()
                                    ))
                                    .with_label(Label::primary(b.file_id, b.span))),
                                DesignationKind::MemberAccess(name) => Err(Diagnostic::error()
                                    .with_message(format!(
                                        "field designator '{}' does not refer to any field",
                                        name
                                    ))
                                    .with_label(Label::primary(b.file_id, b.span))),
                            };
                        }
                        //designations的长度 + 一个root(path[0])
                        keep_num = i + 1;
                    }

                    match &initializer.borrow().kind {
                        InitializerKind::Braced(_) => {
                            while keep_num < path.len() {
                                path.remove(keep_num);
                            }
                        }
                        InitializerKind::Expr(expr) => match &expr.borrow().kind {
                            ExprKind::String { .. } => {
                                //字符串字面量匹配一个数组
                                path.pop();
                            }
                            _ => {}
                        },
                    }

                    let start = initializer.borrow().designation.len() + 1;
                    for node in &path[start..] {
                        initializer
                            .borrow_mut()
                            .designation
                            .push(node.designation.to_designation(file_id, span));
                    }

                    match path.get(1) {
                        Some(DesignationNode {
                            designation: SimpleDesignation::Subscript(index),
                            ..
                        }) => {
                            max_index = max_index.max(*index);

                            match &r#type.borrow().kind {
                                TypeKind::Array {
                                    len_expr: Some(len_expr),
                                    ..
                                } => match &len_expr.borrow().value {
                                    Variant::Int(len) if *len < BigInt::from(max_index + 1) => {
                                        warning(
                                            format!("excess element in array initializer"),
                                            file_id,
                                            span,
                                            vec![],
                                        );
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    self.visit_initializer(
                        Rc::clone(&initializer),
                        if path.len() > 1 {
                            Rc::clone(&path.last().unwrap().r#type)
                        } else {
                            //标量初始化
                            remove_qualifier(Rc::clone(&path.last().unwrap().r#type))
                        },
                    )?;
                    self.next_designation(path.len() - 1, &mut path)?;
                }
            }
            InitializerKind::Expr(expr) => {
                self.visit_expr(Rc::clone(expr))?;

                let mut string_prefix = None;
                let mut string_array = None;

                match &expr.borrow().kind {
                    ExprKind::String { prefix, .. } => {
                        //字符串字面量匹配数组
                        string_prefix = Some(prefix.clone());
                        let expr = expr.borrow();
                        string_array = Some(expr.r#type.borrow().kind.clone());
                        let TypeKind::Array {
                            len_expr: Some(len_expr),
                            ..
                        } = &expr.r#type.borrow().kind
                        else {
                            unreachable!();
                        };
                        match &len_expr.borrow().value {
                            Variant::Int(value) => {
                                max_index = match value.to_usize() {
                                    Some(len) => max_index.max(len - 1),
                                    None => max_index,
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

                if let TypeKind::Array {
                    element_type,
                    len_expr,
                    ..
                } = &node.r#type.borrow().kind
                    && let Some(TypeKind::Array {
                        element_type: string_element_type,
                        len_expr: string_len_expr,
                        ..
                    }) = string_array
                    && let Some(expr_prefix) = string_prefix
                {
                    if !match expr_prefix {
                        EncodePrefix::Default | EncodePrefix::UTF8 => {
                            element_type.borrow().is_char()
                        }
                        _ => {
                            is_compatible(Rc::clone(element_type), Rc::clone(&string_element_type))
                        }
                    } {
                        return Err(Diagnostic::error().with_message(format!(
                                "cannot initializer array of '{}' with array of '{}'(from string literal)",
                                element_type.borrow().to_string(),
                                string_element_type.borrow().to_string()
                            )).with_label(Label::primary(node.file_id, node.span)));
                    }
                    if let Some(len_expr) = len_expr
                        && let Some(string_len_expr) = string_len_expr
                    {
                        if len_expr.borrow().value < string_len_expr.borrow().value {
                            warning(
                                format!("initializer-string is too large"),
                                node.file_id,
                                node.span,
                                vec![],
                            );
                        }
                    }
                } else {
                    match &mut node.kind {
                        InitializerKind::Expr(expr) => {
                            *expr = self.try_implicit_cast(
                                Rc::clone(expr),
                                remove_qualifier(Rc::clone(&r#type)),
                            )?;
                        }
                        _ => {}
                    }
                }
            }
        }

        //补全数组类型
        let file_id = r#type.borrow().file_id;
        let span = r#type.borrow().span;
        let len = max_index + 1;
        match &mut r#type.borrow_mut().kind {
            TypeKind::Array {
                len_expr: len_expr @ None,
                ..
            } => {
                *len_expr = Some(Rc::new(RefCell::new(Expr::new_const_int(
                    file_id,
                    span,
                    len,
                    Rc::new(RefCell::new(Type {
                        kind: TypeKind::ULongLong,
                        ..Type::new(file_id, span)
                    })),
                ))))
            }
            _ => {}
        }

        self.contexts.pop();
        Ok(())
    }

    pub fn visit_designation(
        &mut self,
        designation: &Designation,
    ) -> Result<(), Diagnostic<usize>> {
        match &designation.kind {
            DesignationKind::Subscript(index) => self.visit_expr(Rc::clone(index)),
            DesignationKind::MemberAccess(_) => Ok(()),
        }
    }
}
