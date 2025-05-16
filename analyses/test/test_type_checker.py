from test_common import *


def test_const_and_literal():
    parser = get_parser("const_and_literal.txt")
    parser.tokengen.flag.remove(PPFlag.ALLOW_CONTACT)
    parser.nexttoken()
    for i in (
        BoolLiteral(value=True, type=BoolType()),
        BoolLiteral(value=False, type=BoolType()),
        NullPtrLiteral(type=NullPtrType()),
        IntegerLiteral(value=123, type=IntType()),
        IntegerLiteral(value=0o123, type=IntType()),
        IntegerLiteral(value=0x123, type=IntType()),
        IntegerLiteral(value=0b101, type=IntType()),
        IntegerLiteral(value=4294967296, type=LongLongType()),
        IntegerLiteral(value=123, type=ULongLongType()),
        IntegerLiteral(value=18446744073709551616, type=BitIntType(66)),
        IntegerLiteral(value=18446744073709551616, type=BitIntType(65, False)),
        ImplicitCast(
            expr=StringLiteral(
                value=[49, 50, 51, 230, 136, 145, 0],
                type=ArrayType(CharType(), IntegerLiteral(value=7)),
            )
        ),
        ImplicitCast(
            expr=StringLiteral(
                value=[49, 50, 51, 230, 136, 145, 0],
                type=ArrayType(Char8Type(), IntegerLiteral(value=7)),
            )
        ),
        ImplicitCast(
            expr=StringLiteral(
                value=[49, 50, 51, 25105, 0],
                type=ArrayType(Char16Type(), IntegerLiteral(value=5)),
            )
        ),
        ImplicitCast(
            expr=StringLiteral(
                value=[49, 50, 51, 25105, 0],
                type=ArrayType(Char32Type(), IntegerLiteral(value=5)),
            )
        ),
        ImplicitCast(
            expr=StringLiteral(
                value=[49, 50, 51, 25105, 0],
                type=ArrayType(WCharType(), IntegerLiteral(value=5)),
            )
        ),
        CharLiteral(value=127820, type=Char32Type()),
        CharLiteral(value=16706, type=IntType()),
        CharLiteral(value=29483, type=Char16Type()),
        CharLiteral(value=29483, type=Char32Type()),
    ):
        a = parser.primary_expression()
        symtab = Symtab(a.location)
        a.accept(DeclAnalyzer(symtab))
        a.accept(SymtabFiller(symtab))
        a = a.accept(ConstEvaluater(symtab))
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
    ast = ast.accept(ConstEvaluater(symtab))
    ast = ast.accept(TypeChecker(symtab))

    check_ast(
        ast.items[1],
        ExpressionStmt(
            expr=ArraySubscript(
                array=ImplicitCast(
                    type=PointerType(IntType()),
                    expr=Reference(
                        name="a",
                        type=ArrayType(IntType(), IntegerLiteral(value=3)),
                    ),
                ),
                index=IntegerLiteral(value=2),
                type=IntType(),
            ),
        ),
    )
    check_ast(
        ast.items[2],
        ExpressionStmt(
            expr=ArraySubscript(
                index=ImplicitCast(
                    type=PointerType(IntType()),
                    expr=Reference(
                        name="a",
                        type=ArrayType(IntType(), IntegerLiteral(value=3)),
                    ),
                ),
                array=IntegerLiteral(value=2),
                type=IntType(),
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
    ast = ast.accept(ConstEvaluater(symtab))
    ast = ast.accept(TypeChecker(symtab))

    main_funcdef: FunctionDef = ast.body[1]

    check_ast(
        main_funcdef.body.items[2],
        ExpressionStmt(
            expr=UnaryOperator(
                op=UnaryOpKind.ADDRESS,
                operand=Reference(name="n"),
                type=PointerType(IntType()),
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
                        [CharType()],
                        IntType(),
                    ),
                ),
                type=PointerType(
                    FunctionType(
                        [CharType()],
                        IntType(),
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
                        type=PointerType(IntType()),
                    ),
                    type=IntType(),
                ),
                type=PointerType(IntType()),
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
    ast = ast.accept(ConstEvaluater(symtab))
    ast = ast.accept(TypeChecker(symtab))

    main_funcdef: FunctionDef = ast.body[0]

    record_type: RecordType = main_funcdef.body.items[2].expr.target.type

    check_ast(
        main_funcdef.body.items[1],
        ExpressionStmt(
            expr=MemberRef(
                member_name="x",
                type=IntType(),
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
                type=IntType(),
                target=Reference(name="s", type=record_type),
                is_arrow=False,
            )
        ),
    )


def test_logical_op():
    parser = get_parser("logical_op.txt")
    parser.nexttoken()
    for i in (
        UnaryOperator(op=UnaryOpKind.NOT, type=IntType()),
        BinaryOperator(op=BinOpKind.AND, type=IntType()),
        BinaryOperator(op=BinOpKind.OR, type=IntType()),
    ):
        a: Node = parser.expression()
        symtab = Symtab(a.location)
        a.accept(DeclAnalyzer(symtab))
        a.accept(SymtabFiller(symtab))
        a = a.accept(ConstEvaluater(symtab))
        a = a.accept(TypeChecker(symtab))
        a.accept(DumpVisitor())
        check_ast(a, i)
    assert parser.curtoken().kind == TokenKind.END
