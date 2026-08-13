use crate::{
    ast::TranslationUnit,
    ctype::{RecordKind, Type, TypeKind},
    files,
    legalizer::riscv::Legalizer,
    parse,
    symtab::{Namespace, Symbol, SymbolKind},
    typechecker::TypeChecker,
};
use codespan::Span;
use codespan_reporting::diagnostic::Diagnostic;
use indoc::indoc;
use std::{cell::RefCell, rc::Rc};

//TODO 根据平台决定中间变量的类型

impl Legalizer {
    pub fn has_builtin_function(&self, function_name: &str) -> bool {
        //在编译内建函数的时候, 是共用符号表的
        if let Some(_) = self
            .symtabs
            .last()
            .unwrap()
            .borrow()
            .lookup(Namespace::Ordinary, function_name)
        {
            true
        } else {
            false
        }
    }
    pub fn compile_builtin_function(
        &mut self,
        function_name: &str,
        code: &str,
        new_symbols: Vec<(Namespace, Rc<RefCell<Symbol>>)>,
    ) -> Result<Rc<RefCell<TranslationUnit>>, Diagnostic<usize>> {
        let file_id = files
            .lock()
            .unwrap()
            .add(function_name.to_string(), code.to_string());
        let ast = parse(file_id)?;

        let symtab = &self.symtabs[0]; //最顶层的符号表
        for (namespace, symbol) in new_symbols {
            //假设上层没有添加这个位置
            symbol
                .borrow_mut()
                .declare_locs
                .push((file_id, Span::new(0, 1)));
            symtab.borrow_mut().add(namespace, symbol)?;
        }
        ast.borrow_mut().symtab = Some(Rc::clone(&symtab));

        let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
        type_checker.check(Rc::clone(&ast))?;

        self.builtin_functions
            .insert(function_name.to_string(), ast.clone());

        Ok(ast)
    }

    pub fn builtin_bitint_add(
        &mut self,
        //既是参数类型, 也是返回类型
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(String, Rc<RefCell<Type>>), Diagnostic<usize>> {
        let (_, mut size) = self.get_bitint_info(r#type).unwrap();
        let function_name = format!("__bitint_add_{size}");
        if self.has_builtin_function(&function_name) {
            return Ok((function_name, r#type.clone()));
        }

        let type_name = if let TypeKind::Record { name, .. } = &r#type.borrow().kind {
            name.clone()
        } else {
            unreachable!()
        };
        let mut code = format!(
            indoc! {"struct {type_name} {function_name}(struct {type_name} a, struct {type_name} b);
            struct {type_name} {function_name}(struct {type_name} a, struct {type_name} b)
            {{
                struct {type_name} result;
                unsigned long carry = 0, c1, c2, t1, t2;
            "},
            type_name = type_name,
            function_name = function_name
        );

        let mut i = 0;
        while size > 0 {
            let name = format!("w{i}");

            code += &format!("    t1 = a.{name} + b.{name};\n");
            code += &format!("    c1 = t1 < a.{name};\n");
            code += &format!("    t2 = t1 + carry;\n");
            code += &format!("    c2 = t2 < t1;\n");
            code += &format!("    result.{name} = t2;\n");
            code += &format!("    carry = c1 | c2;\n");

            if size < self.xlen {
                break;
            }
            size -= self.xlen;
            i += 1;
        }
        code += "    return result;\n";
        code += "}\n";

        self.compile_builtin_function(
            &function_name,
            &code,
            vec![(
                Namespace::Tag,
                Rc::new(RefCell::new(Symbol::new(
                    &type_name,
                    SymbolKind::Record {
                        kind: RecordKind::Struct,
                    },
                    r#type,
                ))),
            )],
        )?;

        Ok((function_name, r#type.clone()))
    }

    pub fn builtin_extend(
        &mut self,
        from_type: &Rc<RefCell<Type>>,
        to_type: &Rc<RefCell<Type>>,
        is_zero_extend: bool,
    ) -> Result<(String, Rc<RefCell<Type>>), Diagnostic<usize>> {
        let function_name =
            format!("__bitint_sext_{}_{}", from_type.borrow(), to_type.borrow()).replace(" ", "_");
        if self.has_builtin_function(&function_name) {
            return Ok((function_name, to_type.clone()));
        }

        let mut code;
        let mut new_symbols = vec![];

        if self.was_bitint(from_type) && self.was_bitint(to_type) {
            let from_type_name = if let TypeKind::Record { name, .. } = &from_type.borrow().kind {
                name.clone()
            } else {
                unreachable!()
            };
            let to_type_name = if let TypeKind::Record { name, .. } = &to_type.borrow().kind {
                name.clone()
            } else {
                unreachable!()
            };

            code = format!(
                indoc! {"struct {to_type_name} {function_name}(struct {from_type_name} from);
                struct {to_type_name} {function_name}(struct {from_type_name} from)
                {{
                    struct {to_type_name} to;
                    unsigned long sign,fill=0;
                "},
                from_type_name = from_type_name,
                to_type_name = to_type_name,
                function_name = function_name
            );

            let mut i = 0;
            let (_, mut from_size) = self.get_bitint_info(from_type).unwrap();
            let (_, mut to_size) = self.get_bitint_info(to_type).unwrap();
            assert!(from_size <= to_size);

            while from_size > 0 {
                if from_size < self.xlen && !is_zero_extend {
                    //最后一个成员
                    code += &format!(
                        "    to.w{i}=((long)from.w{i})<<{}ul>>{}ul;\n",
                        self.xlen - from_size,
                        self.xlen - from_size
                    );
                } else {
                    code += &format!("    to.w{i}=from.w{i};\n");
                }

                if from_size < self.xlen {
                    i += 1; //保证不管以哪个路径跳出循环, i都多加了1
                    break;
                }
                from_size -= self.xlen;
                to_size -= self.xlen;
                i += 1;
            }
            i -= 1;

            if !is_zero_extend {
                code += &format!("    sign=(from.w{i}>>{}ul)&1ul;\n", from_size - 1);
                code += &format!(
                    "    fill=sign?{}ul:0ul;\n",
                    match self.xlen {
                        32 => u32::MAX as u64,
                        64 => u64::MAX,
                        _ => unreachable!(),
                    }
                );
            }

            while to_size > 0 {
                code += &format!("    to.w{i}=fill;\n");
                if to_size < self.xlen {
                    break;
                }
                to_size -= self.xlen;
                i += 1;
            }

            code += "    return to;\n";
            code += "}\n";

            new_symbols = vec![
                (
                    Namespace::Tag,
                    Rc::new(RefCell::new(Symbol::new(
                        &from_type_name,
                        SymbolKind::Record {
                            kind: RecordKind::Struct,
                        },
                        from_type,
                    ))),
                ),
                (
                    Namespace::Tag,
                    Rc::new(RefCell::new(Symbol::new(
                        &to_type_name,
                        SymbolKind::Record {
                            kind: RecordKind::Struct,
                        },
                        to_type,
                    ))),
                ),
            ];
        } else if self.was_bitint(from_type) {
            let from_type_name = if let TypeKind::Record { name, .. } = &from_type.borrow().kind {
                name.clone()
            } else {
                unreachable!()
            };
            let (_, from_size) = self.get_bitint_info(from_type).unwrap();
            assert!(from_size <= to_type.borrow().size().unwrap() * 8);

            code = format!(
                indoc! {"{to_type} {function_name}(struct {from_type_name} from);
                {to_type} {function_name}(struct {from_type_name} from)
                {{
                    {to_type} to;
                "},
                from_type_name = from_type_name,
                to_type = to_type.borrow().to_string(),
                function_name = function_name
            );

            if !is_zero_extend {
                code += &format!(
                    "    to=((long)from.w0)<<{}ul>>{}ul;\n",
                    self.xlen - from_size,
                    self.xlen - from_size
                );
            } else {
                code += "    to=from.w0;\n";
            }

            code += "    return to;\n";
            code += "}\n";

            new_symbols = vec![(
                Namespace::Tag,
                Rc::new(RefCell::new(Symbol::new(
                    &from_type_name,
                    SymbolKind::Record {
                        kind: RecordKind::Struct,
                    },
                    from_type,
                ))),
            )];
        } else if self.was_bitint(to_type) {
            let to_type_name = if let TypeKind::Record { name, .. } = &to_type.borrow().kind {
                name.clone()
            } else {
                unreachable!()
            };

            let mut i = 0;
            let from_size = from_type.borrow().size().unwrap() * 8;
            let (_, mut to_size) = self.get_bitint_info(to_type).unwrap();
            assert!(from_size < to_size);

            code = format!(
                indoc! {"struct {to_type_name} {function_name}({from_type} from);
                struct {to_type_name} {function_name}({from_type} from)
                {{
                    struct {to_type_name} to;
                    unsigned long sign=(from>>{}ul)&1ul,fill=0;
                "},
                from_size - 1,
                from_type = from_type.borrow().to_string(),
                to_type_name = to_type_name,
                function_name = function_name
            );

            if !is_zero_extend {
                code += &format!(
                    "    fill=sign?{}ul:0ul;\n",
                    match self.xlen {
                        32 => u32::MAX as u64,
                        64 => u64::MAX,
                        _ => unreachable!(),
                    }
                );
            }

            while to_size > 0 {
                if i == 0 {
                    //如果是符号拓展, 那么from应该是有符号整数, 那么在赋值进行隐式转换时本身就会进行符号扩展, 无需手动进行拓展
                    code += &format!("    to.w{i}=from;\n");
                } else {
                    code += &format!("    to.w{i}=fill;\n");
                }
                if to_size < self.xlen {
                    break;
                }
                to_size -= self.xlen;
                i += 1;
            }

            code += "    return to;\n";
            code += "}\n";

            new_symbols = vec![(
                Namespace::Tag,
                Rc::new(RefCell::new(Symbol::new(
                    &to_type_name,
                    SymbolKind::Record {
                        kind: RecordKind::Struct,
                    },
                    to_type,
                ))),
            )];
        } else {
            unreachable!()
        }

        self.compile_builtin_function(&function_name, &code, new_symbols)?;

        Ok((function_name, to_type.clone()))
    }

    pub fn builtin_bitint_sub(
        &mut self,
        //既是参数类型, 也是返回类型
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(String, Rc<RefCell<Type>>), Diagnostic<usize>> {
        let (_, mut size) = self.get_bitint_info(r#type).unwrap();

        let function_name = format!("__bitint_sub_{size}");
        if self.has_builtin_function(&function_name) {
            return Ok((function_name, r#type.clone()));
        }

        let type_name = if let TypeKind::Record { name, .. } = &r#type.borrow().kind {
            name.clone()
        } else {
            unreachable!()
        };
        let mut code = format!(
            indoc! {"struct {type_name} {function_name}(struct {type_name} a, struct {type_name} b);
            struct {type_name} {function_name}(struct {type_name} a, struct {type_name} b)
            {{
                struct {type_name} result;
                unsigned long borrow = 0, b1, b2, t1, t2;
            "},
            type_name = type_name,
            function_name = function_name
        );

        let mut i = 0;
        while size > 0 {
            let name = format!("w{i}");

            code += &format!("    t1 = a.{name} - b.{name};\n");
            code += &format!("    b1 = a.{name} < b.{name};\n");
            code += &format!("    t2 = t1 - borrow;\n");
            code += &format!("    b2 = t1 < borrow;\n");
            code += &format!("    result.{name} = t2;\n");
            code += &format!("    borrow = b1 | b2;\n");

            if size < self.xlen {
                break;
            }
            size -= self.xlen;
            i += 1;
        }
        code += "    return result;\n";
        code += "}\n";

        self.compile_builtin_function(
            &function_name,
            &code,
            vec![(
                Namespace::Tag,
                Rc::new(RefCell::new(Symbol::new(
                    &type_name,
                    SymbolKind::Record {
                        kind: RecordKind::Struct,
                    },
                    r#type,
                ))),
            )],
        )?;

        Ok((function_name, r#type.clone()))
    }

    pub fn builtin_bitint_neg(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(String, Rc<RefCell<Type>>), Diagnostic<usize>> {
        let (is_unsigned, size) = self.get_bitint_info(r#type).unwrap();

        let function_name = format!("__bitint_neg_{size}");
        if self.has_builtin_function(&function_name) {
            return Ok((function_name, r#type.clone()));
        }

        let code = format!(
            indoc! {"{type} {function_name}({type} a);
            {type} {function_name}({type} a)
            {{
                return (~a)+1;
            }}
            "},
            r#type = if is_unsigned {
                format!("unsigned _BitInt({size})")
            } else {
                format!("_BitInt({size})")
            },
            function_name = function_name
        );

        let ast = self.compile_builtin_function(&function_name, &code, vec![])?;
        Legalizer::new().legalize(&ast)?;

        Ok((function_name, r#type.clone()))
    }

    pub fn builtin_bitint_postfix_incdec(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        is_increase: bool,
    ) -> Result<(String, Rc<RefCell<Type>>), Diagnostic<usize>> {
        let (is_unsigned, size) = self.get_bitint_info(r#type).unwrap();

        let function_name = format!(
            "__bitint_postfix{}_{size}",
            if is_increase { "inc" } else { "dec" }
        );
        if self.has_builtin_function(&function_name) {
            return Ok((function_name, r#type.clone()));
        }

        let code = format!(
            indoc! {"{type} {function_name}({type} *a);
            {type} {function_name}({type} *a)
            {{
                {type} tmp = *a;
                *a = tmp {op} 1;
                return tmp;
            }}
            "},
            r#type = if is_unsigned {
                format!("unsigned _BitInt({size})")
            } else {
                format!("_BitInt({size})")
            },
            function_name = function_name,
            op = if is_increase { "+" } else { "-" }
        );

        let ast = self.compile_builtin_function(&function_name, &code, vec![])?;
        Legalizer::new().legalize(&ast)?;

        Ok((function_name, r#type.clone()))
    }

    pub fn builtin_bitint_prefix_incdec(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        is_increase: bool,
    ) -> Result<(String, Rc<RefCell<Type>>), Diagnostic<usize>> {
        let (is_unsigned, size) = self.get_bitint_info(r#type).unwrap();

        let function_name = format!(
            "__bitint_prefix{}_{size}",
            if is_increase { "inc" } else { "dec" }
        );
        if self.has_builtin_function(&function_name) {
            return Ok((function_name, r#type.clone()));
        }

        let code = format!(
            indoc! {"{type} {function_name}({type} *a);
            {type} {function_name}({type} *a)
            {{
                *a = *a {op} 1;
                return *a;
            }}
            "},
            r#type = if is_unsigned {
                format!("unsigned _BitInt({size})")
            } else {
                format!("_BitInt({size})")
            },
            function_name = function_name,
            op = if is_increase { "+" } else { "-" }
        );

        let ast = self.compile_builtin_function(&function_name, &code, vec![])?;
        Legalizer::new().legalize(&ast)?;

        Ok((function_name, r#type.clone()))
    }

    pub fn builtin_bitint_compare(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        op: &str,
    ) -> Result<(String, Rc<RefCell<Type>>), Diagnostic<usize>> {
        let (is_unsigned, mut size) = self.get_bitint_info(r#type).unwrap();

        let return_type = Rc::new(RefCell::new(Type {
            kind: TypeKind::Bool,
            ..Type::new(r#type.borrow().file_id, r#type.borrow().span)
        }));
        let function_name = format!(
            "__bitint_{}_{}_{size}",
            match op {
                ">" => "gt",
                ">=" => "ge",
                "<" => "lt",
                "<=" => "le",
                _ => unreachable!(),
            },
            if is_unsigned { "unsigned" } else { "signed" }
        );
        if self.has_builtin_function(&function_name) {
            return Ok((function_name, return_type.clone()));
        }

        let type_name = if let TypeKind::Record { name, .. } = &r#type.borrow().kind {
            name.clone()
        } else {
            unreachable!()
        };

        let mut code = format!(
            indoc! {"{return_type} {function_name}(struct {type_name} a, struct {type_name} b);
            {return_type} {function_name}(struct {type_name} a, struct {type_name} b)
            {{
                {return_type} result = true;
            "},
            type_name = type_name,
            return_type = return_type.borrow().to_string(),
            function_name = function_name,
        );

        let mut i = 0;
        while size > 0 {
            code += &format!("    if(!(a.w{i} {op} b.w{i})){{ result=false; goto finished; }}\n");

            if size < self.xlen {
                break;
            }
            size -= self.xlen;
            i += 1;
        }

        code += "finished:\n";

        if !is_unsigned {
            code += &format!("    unsigned long a_sign = (a.w{i}>>{}ul)&1ul;\n", size - 1);
            code += &format!("    unsigned long b_sign = (b.w{i}>>{}ul)&1ul;\n", size - 1);
            code += &format!(
                "    if(a_sign == 0 && b_sign==1) result = {};\n",
                match op {
                    ">" | ">=" => "true",
                    "<" | "<=" => "false",
                    _ => unreachable!(),
                }
            );
            code += &format!(
                "    else if(a_sign == 1 && b_sign==0) result = {};\n",
                match op {
                    ">" | ">=" => "false",
                    "<" | "<=" => "true",
                    _ => unreachable!(),
                }
            );
            code += &format!("    else if(a_sign == 1 && b_sign==1) result = !result;\n");
        }

        code += "    return result;\n";
        code += "}\n";

        self.compile_builtin_function(
            &function_name,
            &code,
            vec![(
                Namespace::Tag,
                Rc::new(RefCell::new(Symbol::new(
                    &type_name,
                    SymbolKind::Record {
                        kind: RecordKind::Struct,
                    },
                    r#type,
                ))),
            )],
        )?;

        Ok((function_name, return_type.clone()))
    }

    pub fn builtin_fp_int_convert(
        &mut self,
        from_type: &Rc<RefCell<Type>>,
        to_type: &Rc<RefCell<Type>>,
    ) -> Result<(String, Rc<RefCell<Type>>), Diagnostic<usize>> {
        let function_name =
            format!("__{}_to_{}", from_type.borrow(), to_type.borrow()).replace(" ", "_");
        if self.has_builtin_function(&function_name) {
            return Ok((function_name, to_type.clone()));
        }

        let code = format!(
            indoc! {"{to_type} {function_name}({from_type} from);
                {to_type} {function_name}({from_type} from)
                {{
                    return (long)from;
                }}
                "},
            from_type = if self.was_bitint(from_type) {
                let (is_unsigned, size) = self.get_bitint_info(from_type).unwrap();
                if is_unsigned {
                    format!("unsigned _BitInt({size})")
                } else {
                    format!("_BitInt({size})")
                }
            } else {
                from_type.borrow().to_string()
            },
            to_type = if self.was_bitint(to_type) {
                let (is_unsigned, size) = self.get_bitint_info(to_type).unwrap();
                if is_unsigned {
                    format!("unsigned _BitInt({size})")
                } else {
                    format!("_BitInt({size})")
                }
            } else {
                to_type.borrow().to_string()
            },
            function_name = function_name
        );

        let mut new_symbols = vec![];

        if self.was_bitint(from_type) {
            let type_name = if let TypeKind::Record { name, .. } = &from_type.borrow().kind {
                name.clone()
            } else {
                unreachable!()
            };

            new_symbols = vec![(
                Namespace::Tag,
                Rc::new(RefCell::new(Symbol::new(
                    &type_name,
                    SymbolKind::Record {
                        kind: RecordKind::Struct,
                    },
                    &from_type,
                ))),
            )];
        }

        if self.was_bitint(to_type) {
            let type_name = if let TypeKind::Record { name, .. } = &to_type.borrow().kind {
                name.clone()
            } else {
                unreachable!()
            };

            new_symbols = vec![(
                Namespace::Tag,
                Rc::new(RefCell::new(Symbol::new(
                    &type_name,
                    SymbolKind::Record {
                        kind: RecordKind::Struct,
                    },
                    &to_type,
                ))),
            )];
        }

        let ast = self.compile_builtin_function(&function_name, &code, new_symbols)?;
        Legalizer::new().legalize(&ast)?;

        Ok((function_name, to_type.clone()))
    }
}
