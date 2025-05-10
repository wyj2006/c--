from Test.Common import *


def test_const_and_literal():
    parser = get_parser("const_and_literal.txt")
    parser.tokengen.flag.remove(PPFlag.ALLOW_CONTACT)
    parser.nexttoken()
    for i in (
        BoolLiteral(value=True, type=BasicType(BasicTypeKind.BOOL)),
        BoolLiteral(value=False, type=BasicType(BasicTypeKind.BOOL)),
        NullPtrLiteral(type=NullPtrType()),
        IntegerLiteral(value=123, type=BasicType(BasicTypeKind.INT)),
        IntegerLiteral(value=0o123, type=BasicType(BasicTypeKind.INT)),
        IntegerLiteral(value=0x123, type=BasicType(BasicTypeKind.INT)),
        IntegerLiteral(value=0b101, type=BasicType(BasicTypeKind.INT)),
        IntegerLiteral(value=4294967296, type=BasicType(BasicTypeKind.LONGLONG)),
        IntegerLiteral(value=123, type=BasicType(BasicTypeKind.ULONGLONG)),
        IntegerLiteral(
            value=18446744073709551616, type=BitIntType(IntegerLiteral(value=66))
        ),
        IntegerLiteral(
            value=18446744073709551616, type=BitIntType(IntegerLiteral(value=65), False)
        ),
        StringLiteral(
            value=[49, 50, 51, 230, 136, 145, 0],
            type=ArrayType(BasicType(BasicTypeKind.CHAR), IntegerLiteral(value=7)),
        ),
        StringLiteral(
            value=[49, 50, 51, 230, 136, 145, 0],
            type=ArrayType(Char8Type(), IntegerLiteral(value=7)),
        ),
        StringLiteral(
            value=[49, 50, 51, 25105, 0],
            type=ArrayType(Char16Type(), IntegerLiteral(value=5)),
        ),
        StringLiteral(
            value=[49, 50, 51, 25105, 0],
            type=ArrayType(Char32Type(), IntegerLiteral(value=5)),
        ),
        StringLiteral(
            value=[49, 50, 51, 25105, 0],
            type=ArrayType(WCharType(), IntegerLiteral(value=5)),
        ),
        CharLiteral(value=127820, type=Char32Type()),
        CharLiteral(value=16706, type=BasicType(BasicTypeKind.INT)),
        CharLiteral(value=29483, type=Char16Type()),
        CharLiteral(value=29483, type=Char32Type()),
    ):
        a = parser.primary_expression()
        symtab = Symtab(a.location)
        a.accept(DeclAnalyzer(symtab))
        a.accept(SymtabFiller(symtab))
        a = a.accept(TypeChecker(symtab))
        a.accept(DumpVisitor())
        check_ast(a, i)
    assert parser.curtoken().kind == TokenKind.END


def test_array_subscript():
    parser = get_parser("array_subscript.txt")
    parser.nexttoken()

    ast: CompoundStmt = CompoundStmt(items=parser.block_item_list())
    ast.location = ast.items[0].location

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))
    ast.accept(SymtabFiller(symtab))
    ast = ast.accept(TypeChecker(symtab))

    check_ast(
        ast.items[1],
        ExpressionStmt(
            expr=ArraySubscript(
                array=ImplicitCast(
                    type=PointerType(BasicType(BasicTypeKind.INT)),
                    expr=Reference(
                        name="a",
                        type=ArrayType(
                            BasicType(BasicTypeKind.INT), IntegerLiteral(value=3)
                        ),
                    ),
                ),
                index=IntegerLiteral(value=2),
                type=BasicType(BasicTypeKind.INT),
            ),
        ),
    )
    check_ast(
        ast.items[2],
        ExpressionStmt(
            expr=ArraySubscript(
                index=ImplicitCast(
                    type=PointerType(BasicType(BasicTypeKind.INT)),
                    expr=Reference(
                        name="a",
                        type=ArrayType(
                            BasicType(BasicTypeKind.INT), IntegerLiteral(value=3)
                        ),
                    ),
                ),
                array=IntegerLiteral(value=2),
                type=BasicType(BasicTypeKind.INT),
            ),
        ),
    )


def test_dereference_address():
    parser = get_parser("dereference_address.txt")
    parser.nexttoken()

    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))
    ast.accept(SymtabFiller(symtab))
    ast = ast.accept(TypeChecker(symtab))

    main_funcdef: FunctionDef = ast.body[1]

    check_ast(
        main_funcdef.body.items[2],
        ExpressionStmt(
            expr=UnaryOperator(
                op=UnaryOpKind.ADDRESS,
                operand=Reference(name="n"),
                type=PointerType(BasicType(BasicTypeKind.INT)),
            )
        ),
    )
    check_ast(
        main_funcdef.body.items[3],
        ExpressionStmt(
            expr=UnaryOperator(
                op=UnaryOpKind.ADDRESS,
                operand=Reference(
                    name="f",
                    type=FunctionType(
                        [BasicType(BasicTypeKind.CHAR)],
                        BasicType(BasicTypeKind.INT),
                    ),
                ),
                type=PointerType(
                    FunctionType(
                        [BasicType(BasicTypeKind.CHAR)],
                        BasicType(BasicTypeKind.INT),
                    )
                ),
            )
        ),
    )
    check_ast(
        main_funcdef.body.items[4],
        ExpressionStmt(
            expr=UnaryOperator(
                op=UnaryOpKind.ADDRESS,
                operand=UnaryOperator(
                    op=UnaryOpKind.DEREFERENCE,
                    operand=Reference(
                        name="p",
                        type=PointerType(BasicType(BasicTypeKind.INT)),
                    ),
                    type=BasicType(BasicTypeKind.INT),
                ),
                type=PointerType(BasicType(BasicTypeKind.INT)),
            )
        ),
    )


def test_member_ref():
    parser = get_parser("member_ref.txt")
    parser.nexttoken()

    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))
    ast.accept(SymtabFiller(symtab))
    ast = ast.accept(TypeChecker(symtab))

    main_funcdef: FunctionDef = ast.body[0]

    record_type: RecordType = main_funcdef.body.items[2].expr.target.type

    check_ast(
        main_funcdef.body.items[1],
        ExpressionStmt(
            expr=MemberRef(
                member_name="x",
                type=BasicType(BasicTypeKind.INT),
                target=Reference(name="p", type=PointerType(record_type)),
                is_arrow=True,
            )
        ),
    )
    check_ast(
        main_funcdef.body.items[2],
        ExpressionStmt(
            expr=MemberRef(
                member_name="x",
                type=BasicType(BasicTypeKind.INT),
                target=Reference(name="s", type=record_type),
                is_arrow=False,
            )
        ),
    )


def test_logical_op():
    parser = get_parser("logical_op.txt")
    parser.nexttoken()
    for i in (
        UnaryOperator(op=UnaryOpKind.NOT, type=BasicType(BasicTypeKind.INT)),
        BinaryOperator(op=BinOpKind.AND, type=BasicType(BasicTypeKind.INT)),
        BinaryOperator(op=BinOpKind.OR, type=BasicType(BasicTypeKind.INT)),
    ):
        a = parser.expression()
        symtab = Symtab(a.location)
        a.accept(DeclAnalyzer(symtab))
        a.accept(SymtabFiller(symtab))
        a = a.accept(TypeChecker(symtab))
        a.accept(DumpVisitor())
        check_ast(a, i)
    assert parser.curtoken().kind == TokenKind.END
