from Common import *


def test_DeclAnalyzer():
    parser = get_parser("decl_analyzer.txt")
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))

    funcdef: FunctionDef = ast.body[0]

    main_func_type: Type = funcdef.func_type
    check_type(
        main_func_type,
        FunctionType(
            [
                BasicType(BasicTypeKind.INT),
                ArrayType(PointerType(BasicType(BasicTypeKind.CHAR)), None),
            ],
            BasicType(BasicTypeKind.INT),
        ),
    )

    # 循环16次
    for i, t in enumerate(
        [
            {"name": "a", "type": BasicType(BasicTypeKind.INT)},
            {"name": "b", "type": PointerType(BasicType(BasicTypeKind.INT))},
            {
                "name": "c",
                "type": ArrayType(
                    ArrayType(BasicType(BasicTypeKind.INT), IntegerLiteral(value="6")),
                    IntegerLiteral(value="5"),
                ),
            },
            {
                "name": "d",
                "type": PointerType(
                    ArrayType(
                        ArrayType(
                            BasicType(BasicTypeKind.INT), IntegerLiteral(value="6")
                        ),
                        IntegerLiteral(value="5"),
                    )
                ),
            },
            {
                "name": "e",
                "type": ArrayType(
                    ArrayType(
                        PointerType(BasicType(BasicTypeKind.INT)),
                        IntegerLiteral(value="6"),
                    ),
                    IntegerLiteral(value="5"),
                ),
            },
            {"name": "f", "type": BasicType(BasicTypeKind.LONGLONG)},
            {"name": "g", "type": BasicType(BasicTypeKind.UNSIGNEDLONGLONG)},
            {"name": "h", "type": BasicType(BasicTypeKind.UNSIGNEDLONGLONG)},
            {"name": "i", "type": BasicType(BasicTypeKind.LONG)},
            {"name": "j", "type": BitIntType(IntegerLiteral(value="8"))},
            {"name": "k", "type": BitIntType(IntegerLiteral(value="9"), signed=False)},
            {
                "name": "l",
                "type": QualifiedType(
                    [TypeQualifier(qualifier=TypeQualifierKind.CONST)],
                    BasicType(BasicTypeKind.INT),
                ),
            },
            {"name": "m", "type": AtomicType(BasicType(BasicTypeKind.INT))},
            {
                "name": "n",
                "type": AtomicType(PointerType(BasicType(BasicTypeKind.INT))),
            },
            {
                "name": "o",
                "type": TypeofType(
                    BinaryOperator(
                        op=BinOpKind.ADD,
                        left=IntegerLiteral(value="1"),
                        right=IntegerLiteral(value="1"),
                    )
                ),
            },
            {
                "name": "p",
                "type": TypeofType(
                    PointerType(PointerType(BasicType(BasicTypeKind.INT)))
                ),
            },
        ]
    ):
        decl_stmt: DeclStmt = funcdef.body.items[i]
        type_or_var_decl: TypeOrVarDecl = decl_stmt.declarators[0]
        assert type_or_var_decl.is_typedef == False
        assert type_or_var_decl.name == t["name"]
        check_type(type_or_var_decl.type, t["type"])

    i += 1
    decl_stmt: DeclStmt = funcdef.body.items[i]
    var_type = QualifiedType(
        [TypeQualifier(qualifier=TypeQualifierKind.CONST)],
        PointerType(
            QualifiedType(
                [TypeQualifier(qualifier=TypeQualifierKind.CONST)],
                BasicType(BasicTypeKind.INT),
            )
        ),
    )
    for type_or_var_decl, t in zip(decl_stmt.declarators, [{"name": "v"}]):
        assert type_or_var_decl.is_typedef == False
        assert type_or_var_decl.name == t["name"]
        check_type(type_or_var_decl.type, var_type)

    i += 1
    decl_stmt: DeclStmt = funcdef.body.items[i]
    type_or_var_decl: TypeOrVarDecl = decl_stmt.declarators[0]
    assert type_or_var_decl.is_typedef == True
    assert type_or_var_decl.name == "A"
    check_type(type_or_var_decl.type, BasicType(BasicTypeKind.INT))

    i += 1
    decl_stmt: DeclStmt = funcdef.body.items[i]
    type_or_var_decl: TypeOrVarDecl = decl_stmt.declarators[0]
    assert type_or_var_decl.is_typedef == False
    assert type_or_var_decl.name == "q"
    check_type(type_or_var_decl.type, TypedefType("A", BasicType(BasicTypeKind.INT)))
    typedef_type = type_or_var_decl.type

    i += 1
    decl_stmt: DeclStmt = funcdef.body.items[i]
    type_or_var_decl: TypeOrVarDecl = decl_stmt.declarators[0]
    assert type_or_var_decl.is_typedef == False
    assert type_or_var_decl.name == "r"
    check_type(
        type_or_var_decl.type,
        QualifiedType(
            [TypeQualifier(qualifier=TypeQualifierKind.CONST)],
            typedef_type,
        ),
    )
    assert id(typedef_type) == id(type_or_var_decl.type.type)

    i += 1
    decl_stmt: DeclStmt = funcdef.body.items[i]
    type_or_var_decl: TypeOrVarDecl = decl_stmt.declarators[0]
    assert type_or_var_decl.is_typedef == False
    assert type_or_var_decl.name == "s"
    check_type(
        type_or_var_decl.type,
        RecordType("struct", "B"),
    )
    record_type = type_or_var_decl.type

    i += 1
    decl_stmt: DeclStmt = funcdef.body.items[i]
    type_or_var_decl: TypeOrVarDecl = decl_stmt.declarators[0]
    assert type_or_var_decl.is_typedef == False
    assert type_or_var_decl.name == "x"
    check_type(
        type_or_var_decl.type,
        record_type,
    )
    assert id(record_type) == id(type_or_var_decl.type)

    i += 1
    decl_stmt: DeclStmt = funcdef.body.items[i]
    type_or_var_decl: TypeOrVarDecl = decl_stmt.declarators[0]
    assert type_or_var_decl.is_typedef == False
    assert type_or_var_decl.name == "t"
    check_type(
        type_or_var_decl.type,
        EnumType("C"),
    )
    enum_type = type_or_var_decl.type

    i += 1
    decl_stmt: DeclStmt = funcdef.body.items[i]
    type_or_var_decl: TypeOrVarDecl = decl_stmt.declarators[0]
    assert type_or_var_decl.is_typedef == False
    assert type_or_var_decl.name == "u"
    check_type(
        type_or_var_decl.type,
        enum_type,
    )
    assert id(enum_type) == id(type_or_var_decl.type)
