use num::{BigInt, BigUint, ToPrimitive};
use pest::Span;

use crate::ast::expr::{Expr, ExprKind};
use crate::ast::{Designation, DesignationKind};
use crate::ctype::TypeKind;
use crate::diagnostic::ErrorKind;
use crate::typechecker::{Context, TypeChecker};
use crate::variant::Variant;
use crate::{
    ast::{Initializer, InitializerKind},
    ctype::Type,
    diagnostic::Error,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
struct DesignationNode<'a> {
    pub index: usize, //该节点是父节点的第几个子节点
    pub r#type: Rc<RefCell<Type<'a>>>,
    pub designation: SimpleDesignation,
}

#[derive(Debug)]
enum SimpleDesignation {
    Subscript(usize),
    MemberAccess(String),
}

impl<'a> DesignationNode<'a> {
    //该节点的第index个子节点, 不存在返回None
    pub fn child(&self, index: usize) -> Result<Option<DesignationNode<'a>>, String> {
        match &self.r#type.borrow().kind {
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
                _ => Err(format!("invalid len")),
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
            TypeKind::Record { members: None, .. } => Err(format!("incomplte type")),
            _ => Ok(None),
        }
    }
}

impl SimpleDesignation {
    pub fn to_designation<'a>(&self, span: Span<'a>) -> Designation<'a> {
        match self {
            SimpleDesignation::Subscript(index) => Designation {
                span,
                kind: DesignationKind::Subscript(Rc::new(RefCell::new(Expr {
                    kind: ExprKind::Integer {
                        base: 10,
                        text: format!("{}", index),
                        type_suffix: vec![],
                    },
                    r#type: Rc::new(RefCell::new(Type {
                        span,
                        attributes: vec![],
                        kind: TypeKind::ULongLong,
                    })),
                    value: Variant::Int(BigInt::from(BigUint::from(*index))),
                    ..Expr::new(span)
                }))),
            },
            SimpleDesignation::MemberAccess(name) => Designation {
                span,
                kind: DesignationKind::MemberAccess(name.clone()),
            },
        }
    }
}

impl<'a> TypeChecker<'a> {
    fn next_designation(
        &self,
        mut i: usize,
        path: &mut Vec<DesignationNode<'a>>,
    ) -> Result<(), String> {
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
        r#type: Rc<RefCell<Type<'a>>>,
    ) -> Result<Vec<DesignationNode<'a>>, String> {
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
        node: Rc<RefCell<Initializer<'a>>>,
        r#type: Rc<RefCell<Type<'a>>>, //这个初始化的类型
    ) -> Result<(), Error<'a>> {
        self.contexts
            .push(Context::Init(node.borrow().kind.clone()));

        let mut node = node.borrow_mut();
        node.r#type = Rc::clone(&r#type);

        let mut max_index: usize = 0; //用于补全数组类型

        match &mut node.kind {
            InitializerKind::Braced(initializers) => {
                let mut path = match self.new_designation_path(Rc::clone(&r#type)) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(Error {
                            span: node.span,
                            kind: ErrorKind::Other(e),
                        });
                    }
                };

                for (i, initializer) in initializers.iter().enumerate() {
                    //标量或者union只能有一个初始化器
                    if i > 0 && (path.len() <= 1 || r#type.borrow().is_union()) {
                        //TODO warning
                        continue;
                    }
                    if initializer.borrow().designation.len() > 0 {
                        //从头开始匹配
                        path = match self.new_designation_path(Rc::clone(&r#type)) {
                            Ok(t) => t,
                            Err(e) => {
                                return Err(Error {
                                    span: node.span,
                                    kind: ErrorKind::Other(e),
                                });
                            }
                        };

                        let span = initializer.borrow().span;
                        let designations = &initializer.borrow().designation;
                        let mut i = 0;
                        //尝试匹配
                        while i < designations.len() && i + 1 < path.len() {
                            let j = i + 1;
                            let a = path[j].designation.to_designation(span);
                            let b = &designations[i];
                            self.visit_designation(b)?;
                            match (&a.kind, &b.kind) {
                                (DesignationKind::Subscript(x), DesignationKind::Subscript(y)) => {
                                    if x.borrow().value != y.borrow().value {
                                        if let Err(e) = self.next_designation(j, &mut path) {
                                            return Err(Error {
                                                span: b.span,
                                                kind: ErrorKind::Other(e),
                                            });
                                        }
                                    } else {
                                        i += 1;
                                    }
                                }
                                (
                                    DesignationKind::MemberAccess(x),
                                    DesignationKind::MemberAccess(y),
                                ) => {
                                    if *x != *y {
                                        if let Err(e) = self.next_designation(j, &mut path) {
                                            return Err(Error {
                                                span: b.span,
                                                kind: ErrorKind::Other(e),
                                            });
                                        }
                                    } else {
                                        i += 1;
                                    }
                                }
                                _ => {
                                    if let Err(e) = self.next_designation(j, &mut path) {
                                        return Err(Error {
                                            span: b.span,
                                            kind: ErrorKind::Other(e),
                                        });
                                    }
                                }
                            }
                        }
                        if i < designations.len() {
                            let b = &designations[i];
                            return match &b.kind {
                                DesignationKind::Subscript(index) => Err(Error {
                                    span: b.span,
                                    kind: ErrorKind::Other(format!(
                                        "array designator index ({}) exceeds array bounds",
                                        index.borrow().unparse()
                                    )),
                                }),
                                DesignationKind::MemberAccess(name) => Err(Error {
                                    span: b.span,
                                    kind: ErrorKind::Other(format!(
                                        "field designator '{}' does not refer to any field",
                                        name
                                    )),
                                }),
                            };
                        }
                        i += 1;
                        while i < path.len() {
                            path.remove(i);
                        }
                    } else {
                        match &initializer.borrow().kind {
                            InitializerKind::Braced(_) => {
                                //只保留顶层的
                                while path.len() >= 3 {
                                    path.remove(2);
                                }
                            }
                            InitializerKind::Expr(_) => {}
                        }

                        initializer.borrow_mut().designation = {
                            let mut designation = vec![];
                            //忽略root(也就是path[0])
                            for node in &path[1..] {
                                designation.push(
                                    node.designation.to_designation(initializer.borrow().span),
                                );
                            }
                            designation
                        };
                    }

                    match path.get(1) {
                        Some(t) => match &t.designation {
                            SimpleDesignation::Subscript(index) => {
                                max_index = max_index.max(*index)
                            }
                            _ => {}
                        },
                        None => {}
                    }

                    self.visit_initializer(
                        Rc::clone(&initializer),
                        Rc::clone(&path.last().unwrap().r#type),
                    )?;
                    if let Err(e) = self.next_designation(path.len() - 1, &mut path) {
                        return Err(Error {
                            span: initializer.borrow().span,
                            kind: ErrorKind::Other(e),
                        });
                    }
                }
            }
            InitializerKind::Expr(expr) => {
                self.visit_expr(Rc::clone(expr))?;

                match &expr.borrow().r#type.borrow().kind {
                    TypeKind::Array {
                        len_expr: Some(len_expr),
                        ..
                    } => match &len_expr.borrow().value {
                        Variant::Int(value) => {
                            max_index = match value.to_usize() {
                                //这里的t是长度不是索引
                                Some(t) => max_index.max(t - 1),
                                None => max_index,
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
                *expr = self.try_implicit_cast(Rc::clone(expr), Rc::clone(&r#type))?;
            }
        }

        //补全数组类型
        let len = max_index + 1;
        match &mut r#type.borrow_mut().kind {
            TypeKind::Array {
                len_expr: len_expr @ None,
                ..
            } => {
                *len_expr = Some(Rc::new(RefCell::new(Expr {
                    kind: ExprKind::Integer {
                        base: 10,
                        text: format!("{len}"),
                        type_suffix: vec![],
                    },
                    r#type: Rc::new(RefCell::new(Type {
                        span: node.span,
                        attributes: vec![],
                        kind: TypeKind::ULongLong,
                    })),
                    value: Variant::Int(BigInt::from(BigUint::from(len))),
                    ..Expr::new(node.span)
                })))
            }
            _ => {}
        }

        self.contexts.pop();
        Ok(())
    }

    pub fn visit_designation(&mut self, designation: &Designation<'a>) -> Result<(), Error<'a>> {
        match &designation.kind {
            DesignationKind::Subscript(index) => self.visit_expr(Rc::clone(index)),
            DesignationKind::MemberAccess(_) => Ok(()),
        }
    }
}
