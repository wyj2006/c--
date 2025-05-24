from test_common import *


def test_eval_enum_const():
    parser = get_parser("eval_enum_const.txt")
    parser.nexttoken()

    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))
    ast.accept(SymtabFiller(symtab))
    ast = ast.accept(ConstEvaluater(symtab))
    ast = ast.accept(TypeChecker(symtab))

    enum_type: EnumType = symtab.lookup("A", TAG_NAMES)

    assert enum_type.enumerators["Y"].value == 2147483647
    assert isinstance(enum_type.enumerators["Y"].value.type, UIntType)
    assert enum_type.enumerators["X"].value == 2147483648
    assert isinstance(enum_type.enumerators["X"].value.type, UIntType)


def test_eval_array_size():
    parser = get_parser("eval_array_size.txt")
    parser.nexttoken()

    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))
    ast.accept(SymtabFiller(symtab))
    ast = ast.accept(ConstEvaluater(symtab))
    ast = ast.accept(TypeChecker(symtab))

    var: Object = symtab.children[0].lookup("a")
    array_type: ArrayType = var.type
    assert array_type.size_expr.value == 3

    var: Object = symtab.children[0].lookup("b")
    array_type: ArrayType = var.type
    assert array_type.size_expr.value == 6

    var: Object = symtab.children[0].lookup("c")
    array_type: ArrayType = var.type
    assert array_type.size_expr.value == 9


def test_eval_const():
    parser = get_parser("eval_const.txt")
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
        BinaryOperator(),  # TODO
    ):
        a = parser.expression()
        symtab = Symtab(a.location)
        a.accept(DeclAnalyzer(symtab))
        a.accept(SymtabFiller(symtab))
        a = a.accept(ConstEvaluater(symtab))
        a.accept(DumpVisitor())
        check_ast(a, i)
    assert parser.curtoken().kind == TokenKind.END
