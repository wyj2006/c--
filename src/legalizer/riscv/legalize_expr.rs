use crate::{
    ast::expr::{BinOpKind, CastMethod, Expr, ExprKind, GenericAssoc, UnaryOpKind},
    ctype::{Type, TypeKind, cast::try_implicit_cast},
    legalizer::riscv::Legalizer,
};
use codespan_reporting::diagnostic::Diagnostic;
use num::{BigRational, Zero};
use std::{cell::RefCell, rc::Rc};

impl Legalizer {
    pub fn visit_expr(&mut self, node: &Rc<RefCell<Expr>>) -> Result<(), Diagnostic<usize>> {
        let mut new_expr = None;
        let file_id = node.borrow().file_id;
        let span = node.borrow().span;

        self.legalize_type(&node.borrow().r#type)?;
        let node_type = node.borrow().r#type.clone();

        match &node.borrow().kind {
            ExprKind::Cast {
                target,
                decls,
                method,
                ..
            } => {
                for decl in decls {
                    self.visit_declaration(decl)?;
                }
                self.visit_expr(target)?;

                match method {
                    CastMethod::ComplexExtend | CastMethod::ComplexTrunc => {
                        let part_type = self.get_complex_part_type(&node_type).unwrap();
                        let real_part = self.extract_member(target, "real").unwrap();
                        let imag_part = self.extract_member(target, "imag").unwrap();

                        new_expr = Some(self.make_compound(
                            file_id,
                            span,
                            vec![
                                (
                                    "real".to_string(),
                                    try_implicit_cast(real_part, part_type.clone())?,
                                ),
                                (
                                    "imag".to_string(),
                                    try_implicit_cast(imag_part, part_type.clone())?,
                                ),
                            ],
                            node_type,
                        ));
                    }
                    CastMethod::ComplexToFloat => {
                        let real_part = self.extract_member(target, "real").unwrap();
                        new_expr = Some(try_implicit_cast(real_part, node_type.clone())?)
                    }
                    CastMethod::FloatToComplex => {
                        let part_type = self.get_complex_part_type(&node_type).unwrap();
                        new_expr = Some(self.make_compound(
                            file_id,
                            span,
                            vec![
                                (
                                    "real".to_string(),
                                    try_implicit_cast(target.clone(), part_type.clone())?,
                                ),
                                (
                                    "imag".to_string(),
                                    Rc::new(RefCell::new(Expr::new_const_rational(
                                        file_id,
                                        span,
                                        BigRational::zero(),
                                        part_type,
                                    ))),
                                ),
                            ],
                            node_type,
                        ));
                    }
                    CastMethod::FloatToSInt
                    | CastMethod::FloatToUInt
                    | CastMethod::SIntToFloat
                    | CastMethod::UIntToFloat
                        if self.was_bitint(&node_type)
                            || self.was_bitint(&target.borrow().r#type) =>
                    {
                        let (function_name, return_type) =
                            self.builtin_fp_int_convert(&target.borrow().r#type, &node_type)?;
                        new_expr = Some(self.make_builtin_call(
                            file_id,
                            span,
                            &function_name,
                            vec![target.clone()],
                            &return_type,
                        ));
                    }
                    CastMethod::IntToPtr | CastMethod::IntTrunc
                        if !self.was_bitint(&node_type) //这个条件属于CastMethod::IntTrunc, 但对CastMethod::IntToPtr也适用
                            && self.was_bitint(&target.borrow().r#type) =>
                    {
                        target.replace(
                            (*self.extract_member(target, "w0").unwrap().borrow()).clone(),
                        );
                    }
                    CastMethod::PtrToInt | CastMethod::IntTrunc
                        if self.was_bitint(&node_type)
                        //这个条件属于CastMethod::IntTrunc, 但对CastMethod::PtrToInt也适用
                            && !self.was_bitint(&target.borrow().r#type) =>
                    {
                        let (_, mut size) = self.get_bitint_info(&node_type).unwrap();
                        let member_type = Rc::new(RefCell::new(Type {
                            kind: TypeKind::ULong,
                            ..Type::new(file_id, span)
                        }));
                        let mut members = vec![];
                        let mut i = 0;
                        //在CastMethod::IntTrunc情况下, 这个循环一般只会进入一次
                        while size > 0 {
                            let name = format!("w{i}");

                            if i == 0 {
                                members.push((
                                    name,
                                    try_implicit_cast(target.clone(), member_type.clone())?,
                                ));
                            } else {
                                members.push((
                                    name,
                                    Rc::new(RefCell::new(Expr::new_const_int(
                                        file_id,
                                        span,
                                        0,
                                        member_type.clone(),
                                    ))),
                                ));
                            }

                            if size < self.xlen {
                                break;
                            }
                            size -= self.xlen;

                            i += 1;
                        }

                        new_expr =
                            Some(self.make_compound(file_id, span, members, node_type.clone()));
                    }
                    CastMethod::IntTrunc
                        if self.was_bitint(&node_type)
                            && self.was_bitint(&target.borrow().r#type) =>
                    {
                        let (_, mut size) = self.get_bitint_info(&node_type).unwrap();
                        let mut members = vec![];
                        let mut i = 0;

                        while size > 0 {
                            let name = format!("w{i}");

                            members.push((
                                name.clone(),
                                //两个bitint的成员类型是一样的, 无需隐式转换
                                self.extract_member(target, &name).unwrap(),
                            ));

                            if size < self.xlen {
                                break;
                            }
                            size -= self.xlen;

                            i += 1;
                        }

                        new_expr =
                            Some(self.make_compound(file_id, span, members, node_type.clone()));
                    }
                    CastMethod::SignedExtend | CastMethod::ZeroExtand
                        if self.was_bitint(&node_type)
                            || self.was_bitint(&target.borrow().r#type) =>
                    {
                        let (function_name, return_type) = self.builtin_extend(
                            &target.borrow().r#type,
                            &node_type,
                            matches!(method, CastMethod::ZeroExtand),
                        )?;
                        new_expr = Some(self.make_builtin_call(
                            file_id,
                            span,
                            &function_name,
                            vec![target.clone()],
                            &return_type,
                        ));
                    }
                    CastMethod::ToBool if self.was_complex(&target.borrow().r#type) => {
                        let part_type =
                            self.get_complex_part_type(&target.borrow().r#type).unwrap();
                        let zero = Rc::new(RefCell::new(Expr::new_const_rational(
                            file_id,
                            span,
                            BigRational::zero(),
                            part_type.clone(),
                        )));

                        let real_part = self.extract_member(target, "real");
                        let imag_part = self.extract_member(target, "imag");

                        new_expr = Some(self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::And,
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Neq,
                                real_part,
                                Some(zero.clone()),
                                Some(node_type.clone()), //bool
                            )),
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Neq,
                                imag_part,
                                Some(zero.clone()),
                                Some(node_type.clone()), //bool
                            )),
                            Some(node_type.clone()), //bool
                        ));
                    }
                    CastMethod::ToBool if self.was_bitint(&target.borrow().r#type) => {
                        let (_, mut size) = self.get_bitint_info(&target.borrow().r#type).unwrap();
                        let zero = Rc::new(RefCell::new(Expr::new_const_int(
                            file_id,
                            span,
                            0,
                            Rc::new(RefCell::new(Type {
                                //TODO 根据平台决定
                                kind: TypeKind::ULong,
                                ..Type::new(file_id, span)
                            })),
                        )));
                        let mut result = None;

                        let mut i = 0;
                        while size > 0 {
                            let name = format!("w{i}");

                            let a = self.extract_member(target, &name);
                            let c = self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Neq,
                                a,
                                Some(zero.clone()),
                                Some(node_type.clone()), //bool
                            );

                            result = Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::And,
                                result,
                                Some(c),
                                Some(node_type.clone()), //bool
                            ));

                            if size < self.xlen {
                                break;
                            }
                            size -= self.xlen;

                            i += 1;
                        }

                        new_expr = result;
                    }
                    _ => {}
                }
            }
            ExprKind::BinOp { op, left, right } => {
                self.visit_expr(left)?;
                self.visit_expr(right)?;
                match op {
                    BinOpKind::Add
                        if self.was_complex(&left.borrow().r#type)
                            || self.was_complex(&right.borrow().r#type) =>
                    {
                        let left_real = self.extract_member(left, "real");
                        let left_imag = self.extract_member(left, "imag");
                        let right_real = self.extract_member(right, "real");
                        let right_imag = self.extract_member(right, "imag");

                        let real_part = self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::Add,
                            left_real,
                            right_real,
                            None,
                        );
                        let imag_part = self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::Add,
                            left_imag,
                            right_imag,
                            None,
                        );

                        //构造 (struct complex){.real=a.real+b.real, .imag=a.imag+b.imag}
                        new_expr = Some(self.make_compound(
                            file_id,
                            span,
                            vec![
                                ("real".to_string(), real_part),
                                ("imag".to_string(), imag_part),
                            ],
                            node_type,
                        ));
                    }
                    BinOpKind::Add
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        let (function_name, return_type) = self.builtin_bitint_add(&node_type)?;
                        new_expr = Some(self.make_builtin_call(
                            file_id,
                            span,
                            &function_name,
                            vec![left.clone(), right.clone()],
                            &return_type,
                        ));
                    }
                    BinOpKind::Sub
                        if self.was_complex(&left.borrow().r#type)
                            || self.was_complex(&right.borrow().r#type) =>
                    {
                        let left_real = self.extract_member(left, "real");
                        let left_imag = self.extract_member(left, "imag");
                        let right_real = self.extract_member(right, "real");
                        let right_imag = self.extract_member(right, "imag");

                        let real_part = self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::Sub,
                            left_real,
                            right_real,
                            None,
                        );
                        let imag_part = self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::Sub,
                            left_imag,
                            right_imag,
                            None,
                        );

                        new_expr = Some(self.make_compound(
                            file_id,
                            span,
                            vec![
                                ("real".to_string(), real_part),
                                ("imag".to_string(), imag_part),
                            ],
                            node_type,
                        ));
                    }
                    BinOpKind::Sub
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        let (function_name, return_type) = self.builtin_bitint_sub(&node_type)?;
                        new_expr = Some(self.make_builtin_call(
                            file_id,
                            span,
                            &function_name,
                            vec![left.clone(), right.clone()],
                            &return_type,
                        ));
                    }
                    BinOpKind::Mul
                        if self.was_complex(&left.borrow().r#type)
                            || self.was_complex(&right.borrow().r#type) =>
                    {
                        let left_real = self.extract_member(left, "real");
                        let left_imag = self.extract_member(left, "imag");
                        let right_real = self.extract_member(right, "real");
                        let right_imag = self.extract_member(right, "imag");

                        let real_part = self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::Sub,
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Mul,
                                left_real.clone(),
                                right_real.clone(),
                                None,
                            )),
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Mul,
                                left_imag.clone(),
                                right_imag.clone(),
                                None,
                            )),
                            None,
                        );
                        let imag_part = self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::Add,
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Mul,
                                left_real.clone(),
                                right_imag.clone(),
                                None,
                            )),
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Mul,
                                right_real.clone(),
                                left_imag.clone(),
                                None,
                            )),
                            None,
                        );

                        new_expr = Some(self.make_compound(
                            file_id,
                            span,
                            vec![
                                ("real".to_string(), real_part),
                                ("imag".to_string(), imag_part),
                            ],
                            node_type,
                        ));
                    }
                    BinOpKind::Mul
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        //TODO Mul
                        todo!()
                    }
                    BinOpKind::Div
                        if self.was_complex(&left.borrow().r#type)
                            || self.was_complex(&right.borrow().r#type) =>
                    {
                        let left_real = self.extract_member(left, "real");
                        let left_imag = self.extract_member(left, "imag");
                        let right_real = self.extract_member(right, "real");
                        let right_imag = self.extract_member(right, "imag");

                        let real_part = self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::Div,
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Add,
                                Some(self.make_binary_op(
                                    file_id,
                                    span,
                                    BinOpKind::Mul,
                                    left_real.clone(),
                                    right_real.clone(),
                                    None,
                                )),
                                Some(self.make_binary_op(
                                    file_id,
                                    span,
                                    BinOpKind::Mul,
                                    left_imag.clone(),
                                    right_imag.clone(),
                                    None,
                                )),
                                None,
                            )),
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Add,
                                Some(self.make_binary_op(
                                    file_id,
                                    span,
                                    BinOpKind::Mul,
                                    right_real.clone(),
                                    right_real.clone(),
                                    None,
                                )),
                                Some(self.make_binary_op(
                                    file_id,
                                    span,
                                    BinOpKind::Mul,
                                    right_imag.clone(),
                                    right_imag.clone(),
                                    None,
                                )),
                                None,
                            )),
                            None,
                        );
                        let imag_part = self.make_binary_op(
                            file_id,
                            span,
                            BinOpKind::Div,
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Sub,
                                Some(self.make_binary_op(
                                    file_id,
                                    span,
                                    BinOpKind::Mul,
                                    right_real.clone(),
                                    left_imag.clone(),
                                    None,
                                )),
                                Some(self.make_binary_op(
                                    file_id,
                                    span,
                                    BinOpKind::Mul,
                                    left_real.clone(),
                                    right_imag.clone(),
                                    None,
                                )),
                                None,
                            )),
                            Some(self.make_binary_op(
                                file_id,
                                span,
                                BinOpKind::Add,
                                Some(self.make_binary_op(
                                    file_id,
                                    span,
                                    BinOpKind::Mul,
                                    right_real.clone(),
                                    right_real.clone(),
                                    None,
                                )),
                                Some(self.make_binary_op(
                                    file_id,
                                    span,
                                    BinOpKind::Mul,
                                    right_imag.clone(),
                                    right_imag.clone(),
                                    None,
                                )),
                                None,
                            )),
                            None,
                        );

                        new_expr = Some(self.make_compound(
                            file_id,
                            span,
                            vec![
                                ("real".to_string(), real_part),
                                ("imag".to_string(), imag_part),
                            ],
                            node_type,
                        ));
                    }
                    BinOpKind::Div
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        //TODO Div
                        todo!()
                    }
                    BinOpKind::LShift | BinOpKind::RShift
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        //TODO L/RShift
                        todo!()
                    }
                    BinOpKind::LShift | BinOpKind::RShift
                        if self.was_bitint(&left.borrow().r#type)
                            && !self.was_bitint(&right.borrow().r#type) =>
                    {
                        let (is_unsigned, mut size) =
                            self.get_bitint_info(&left.borrow().r#type).unwrap();
                        let mut members = vec![];

                        let mut i = 0;
                        while size > 0 {
                            let name = format!("w{i}");

                            let a = match self.extract_member(left, &name) {
                                Some(t) if size < self.xlen && !is_unsigned => {
                                    //对于LShift来说这个转换并不影响
                                    //在对a.wi右移后应该还需要隐式转换回原来的无符号类型,
                                    //但由于无符号和有符号类型长度一样, 所以什么都不用做
                                    Some(try_implicit_cast(
                                        t,
                                        Rc::new(RefCell::new(Type {
                                            //TODO 根据平台决定
                                            kind: TypeKind::Long,
                                            ..Type::new(file_id, span)
                                        })),
                                    )?)
                                }
                                t => t,
                            };

                            let t = self.make_binary_op(
                                file_id,
                                span,
                                *op,
                                a,
                                Some(right.clone()),
                                None,
                            );

                            members.push((name, t));

                            if size < self.xlen {
                                break;
                            }
                            size -= self.xlen;

                            i += 1;
                        }

                        new_expr = Some(self.make_compound(file_id, span, members, node_type));
                    }
                    BinOpKind::LShift | BinOpKind::RShift
                        if !self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        //相当于把right截断后再移动, 因为left的长度最大是64, 如果right很大, 那么截不截断对结果没有影响
                        right
                            .replace((*self.extract_member(right, "w0").unwrap().borrow()).clone());
                    }
                    BinOpKind::Mod
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        //TODO Mod
                        todo!()
                    }
                    BinOpKind::Lt | BinOpKind::Le | BinOpKind::Gt | BinOpKind::Ge
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        let (function_name, return_type) = self.builtin_bitint_compare(
                            &left.borrow().r#type,
                            match op {
                                BinOpKind::Lt => "<",
                                BinOpKind::Le => "<=>",
                                BinOpKind::Gt => ">",
                                BinOpKind::Ge => ">=",
                                _ => unreachable!(),
                            },
                        )?;

                        new_expr = Some(self.make_builtin_call(
                            file_id,
                            span,
                            &function_name,
                            vec![left.clone(), right.clone()],
                            &return_type,
                        ));
                    }
                    BinOpKind::Eq | BinOpKind::Neq
                        if self.was_complex(&left.borrow().r#type)
                            || self.was_complex(&right.borrow().r#type) =>
                    {
                        let left_real = self.extract_member(left, "real");
                        let left_imag = self.extract_member(left, "imag");
                        let right_real = self.extract_member(right, "real");
                        let right_imag = self.extract_member(right, "imag");

                        let real_part = self.make_binary_op(
                            file_id,
                            span,
                            *op,
                            left_real,
                            right_real,
                            Some(node_type.clone()), //bool
                        );
                        let imag_part = self.make_binary_op(
                            file_id,
                            span,
                            *op,
                            left_imag,
                            right_imag,
                            Some(node_type.clone()), //bool
                        );

                        new_expr = Some(self.make_binary_op(
                            file_id,
                            span,
                            if let BinOpKind::Eq = op {
                                BinOpKind::And
                            } else {
                                BinOpKind::Or
                            },
                            Some(real_part),
                            Some(imag_part),
                            Some(node_type.clone()), //bool
                        ));
                    }
                    BinOpKind::Eq | BinOpKind::Neq
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        let (_, mut size) = self.get_bitint_info(&left.borrow().r#type).unwrap();

                        let mut result = None;

                        let mut i = 0;
                        while size > 0 {
                            let name = format!("w{i}");

                            let a = self.extract_member(left, &name);
                            let b = self.extract_member(right, &name);

                            let c = self.make_binary_op(
                                file_id,
                                span,
                                *op,
                                a,
                                b,
                                Some(node_type.clone()), //bool
                            );

                            result = Some(self.make_binary_op(
                                file_id,
                                span,
                                if let BinOpKind::Eq = op {
                                    BinOpKind::And
                                } else {
                                    BinOpKind::Or
                                },
                                result,
                                Some(c),
                                Some(node_type.clone()), //bool
                            ));

                            if size < self.xlen {
                                break;
                            }
                            size -= self.xlen;

                            i += 1;
                        }

                        new_expr = result;
                    }
                    BinOpKind::BitAnd | BinOpKind::BitOr | BinOpKind::BitXOr
                        if self.was_bitint(&left.borrow().r#type)
                            && self.was_bitint(&right.borrow().r#type) =>
                    {
                        let (_, mut size) = self.get_bitint_info(&left.borrow().r#type).unwrap();
                        let mut members = vec![];

                        let mut i = 0;
                        while size > 0 {
                            let name = format!("w{i}");

                            let a = self.extract_member(left, &name);
                            let b = self.extract_member(right, &name);

                            let t = self.make_binary_op(file_id, span, *op, a, b, None);

                            members.push((name, t));

                            if size < self.xlen {
                                break;
                            }
                            size -= self.xlen;

                            i += 1;
                        }

                        new_expr = Some(self.make_compound(file_id, span, members, node_type));
                    }
                    _ => {}
                }
            }
            ExprKind::UnaryOp { op, operand } => {
                self.visit_expr(operand)?;
                match op {
                    UnaryOpKind::BitNot if self.was_bitint(&operand.borrow().r#type) => {
                        let (_, mut size) = self.get_bitint_info(&operand.borrow().r#type).unwrap();
                        let mut members = vec![];

                        let mut i = 0;
                        while size > 0 {
                            let name = format!("w{i}");

                            let a = self.extract_member(operand, &name).unwrap();
                            let t = self.make_unary_op(file_id, span, *op, &a, None);

                            members.push((name, t));

                            if size < self.xlen {
                                break;
                            }
                            size -= self.xlen;

                            i += 1;
                        }

                        new_expr = Some(self.make_compound(file_id, span, members, node_type));
                    }
                    UnaryOpKind::Negative if self.was_complex(&operand.borrow().r#type) => {
                        let operand_real = self.extract_member(operand, "real").unwrap();
                        let operand_imag = self.extract_member(operand, "imag").unwrap();

                        let real_part = self.make_unary_op(
                            file_id,
                            span,
                            UnaryOpKind::Negative,
                            &operand_real,
                            None,
                        );
                        let imag_part = self.make_unary_op(
                            file_id,
                            span,
                            UnaryOpKind::Negative,
                            &operand_imag,
                            None,
                        );

                        new_expr = Some(self.make_compound(
                            file_id,
                            span,
                            vec![
                                ("real".to_string(), real_part),
                                ("imag".to_string(), imag_part),
                            ],
                            node_type,
                        ));
                    }
                    UnaryOpKind::Negative if self.was_bitint(&operand.borrow().r#type) => {
                        let (function_name, return_type) =
                            self.builtin_bitint_neg(&operand.borrow().r#type)?;
                        new_expr = Some(self.make_builtin_call(
                            file_id,
                            span,
                            &function_name,
                            vec![operand.clone()],
                            &return_type,
                        ));
                    }
                    UnaryOpKind::PostfixDec
                    | UnaryOpKind::PostfixInc
                    | UnaryOpKind::PrefixDec
                    | UnaryOpKind::PrefixInc
                        if self.was_bitint(&operand.borrow().r#type) =>
                    {
                        let (function_name, return_type) = match op {
                            UnaryOpKind::PostfixDec => {
                                self.builtin_bitint_postfix_incdec(&operand.borrow().r#type, false)?
                            }
                            UnaryOpKind::PostfixInc => {
                                self.builtin_bitint_postfix_incdec(&operand.borrow().r#type, true)?
                            }
                            UnaryOpKind::PrefixDec => {
                                self.builtin_bitint_prefix_incdec(&operand.borrow().r#type, false)?
                            }
                            UnaryOpKind::PrefixInc => {
                                self.builtin_bitint_prefix_incdec(&operand.borrow().r#type, true)?
                            }
                            _ => unreachable!(),
                        };
                        new_expr = Some(self.make_builtin_call(
                            file_id,
                            span,
                            &function_name,
                            vec![self.make_unary_op(
                                file_id,
                                span,
                                UnaryOpKind::AddressOf,
                                operand,
                                Some(Rc::new(RefCell::new(Type {
                                    kind: TypeKind::Pointer(operand.borrow().r#type.clone()),
                                    ..Type::new(file_id, span)
                                }))),
                            )],
                            &return_type,
                        ));
                    }
                    _ => {}
                }
            }
            ExprKind::CompoundLiteral {
                decls, initializer, ..
            } => {
                for decl in decls {
                    self.visit_declaration(decl)?;
                }
                self.visit_initializer(initializer)?;
            }
            ExprKind::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                self.visit_expr(condition)?;
                self.visit_expr(true_expr)?;
                self.visit_expr(false_expr)?;
            }
            ExprKind::FunctionCall { target, arguments } => {
                for arg in arguments {
                    self.visit_expr(arg)?;
                }
                self.visit_expr(target)?;
            }
            ExprKind::GenericSelection {
                control_expr,
                assocs,
            } => {
                self.visit_expr(control_expr)?;
                for assoc in assocs {
                    let GenericAssoc {
                        is_selected: true,
                        expr,
                        ..
                    } = &*assoc.borrow()
                    else {
                        continue;
                    };
                    self.visit_expr(expr)?;
                }
            }
            ExprKind::MemberAccess { target, .. } => {
                self.visit_expr(target)?;
            }
            ExprKind::Subscript { target, index } => {
                self.visit_expr(target)?;
                self.visit_expr(index)?;
            }
            _ => {}
        }

        if let Some(new_expr) = new_expr {
            node.replace((*new_expr.borrow()).clone());
        }

        Ok(())
    }
}
