from Test.Common import *


def test_basic():
    parser = get_parser("basic.txt")
    parser.nexttoken()
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))

    check_ast(
        ast,
        TranslationUnit(
            body=[
                DeclStmt(declarators=[TypeOrVarDecl(name="a", type=IntType())]),
                DeclStmt(
                    declarators=[TypeOrVarDecl(name="b", type=PointerType(IntType()))]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="c",
                            type=ArrayType(
                                ArrayType(
                                    IntType(),
                                    IntegerLiteral(value="6"),
                                ),
                                IntegerLiteral(value="5"),
                            ),
                        )
                    ]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="d",
                            type=PointerType(
                                ArrayType(
                                    ArrayType(
                                        IntType(),
                                        IntegerLiteral(value="6"),
                                    ),
                                    IntegerLiteral(value="5"),
                                )
                            ),
                        )
                    ]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="e",
                            type=ArrayType(
                                ArrayType(
                                    PointerType(IntType()),
                                    IntegerLiteral(value="6"),
                                ),
                                IntegerLiteral(value="5"),
                            ),
                        )
                    ]
                ),
                DeclStmt(declarators=[TypeOrVarDecl(name="f", type=LongLongType())]),
                DeclStmt(declarators=[TypeOrVarDecl(name="g", type=ULongLongType())]),
                DeclStmt(declarators=[TypeOrVarDecl(name="h", type=ULongLongType())]),
                DeclStmt(declarators=[TypeOrVarDecl(name="i", type=LongType())]),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="j", type=BitIntType(IntegerLiteral(value="8"))
                        )
                    ]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="k",
                            type=BitIntType(IntegerLiteral(value="9"), signed=False),
                        )
                    ]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="l",
                            type=QualifiedType(
                                [TypeQualifier(qualifier=TypeQualifierKind.CONST)],
                                IntType(),
                            ),
                        )
                    ]
                ),
                DeclStmt(
                    declarators=[TypeOrVarDecl(name="m", type=AtomicType(IntType()))]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="n",
                            type=AtomicType(PointerType(IntType())),
                        )
                    ]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="o",
                            type=TypeofType(
                                BinaryOperator(
                                    op=BinOpKind.ADD,
                                    left=IntegerLiteral(value="1"),
                                    right=IntegerLiteral(value="1"),
                                )
                            ),
                        )
                    ]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="p",
                            type=TypeofType(PointerType(PointerType(IntType()))),
                        )
                    ]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="v",
                            type=QualifiedType(
                                [TypeQualifier(qualifier=TypeQualifierKind.CONST)],
                                PointerType(
                                    QualifiedType(
                                        [
                                            TypeQualifier(
                                                qualifier=TypeQualifierKind.CONST
                                            )
                                        ],
                                        IntType(),
                                    )
                                ),
                            ),
                        ),
                        TypeOrVarDecl(
                            name="w",
                            type=QualifiedType(
                                [TypeQualifier(qualifier=TypeQualifierKind.CONST)],
                                IntType(),
                            ),
                        ),
                    ]
                ),
                FunctionDef(
                    func_name="main",
                    func_type=FunctionType(
                        [
                            IntType(),
                            ArrayType(PointerType(CharType()), None),
                        ],
                        IntType(),
                    ),
                ),
            ]
        ),
    )


def test_symtab_related():
    parser = get_parser("symtab_related.txt")
    parser.nexttoken()
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))

    typedef_type = symtab.lookup("A")
    struct_type = symtab.lookup("B", TAG_NAMES)
    enum_type = symtab.lookup("C", TAG_NAMES)

    assert ast.body[0].declarators[0].is_typedef
    assert ast.body[1].declarators[0].type is typedef_type
    assert ast.body[2].declarators[0].type.type is typedef_type
    assert (
        ast.body[3].specifiers[0].members_declaration[0].declarators[0].type
        is typedef_type
    )
    assert ast.body[4].declarators[0].type is struct_type
    assert ast.body[6].declarators[0].type is enum_type

    check_ast(
        ast,
        TranslationUnit(
            body=[
                DeclStmt(declarators=[TypeOrVarDecl(name="A", type=IntType())]),  # 0
                DeclStmt(declarators=[TypeOrVarDecl(name="q", type=typedef_type)]),  # 1
                DeclStmt(  # 2
                    declarators=[
                        TypeOrVarDecl(
                            name="r",
                            type=QualifiedType(
                                [TypeQualifier(qualifier=TypeQualifierKind.CONST)],
                                typedef_type,
                            ),
                        )
                    ]
                ),
                DeclStmt(  # 3
                    specifiers=[
                        RecordDecl(
                            struct_or_union="struct",
                            name="B",
                            type=struct_type,
                            members_declaration=[
                                FieldDecl(
                                    declarators=[
                                        MemberDecl(name="a", type=typedef_type)
                                    ]
                                )
                            ],
                        )
                    ],
                    declarators=[TypeOrVarDecl(name="s", type=struct_type)],
                ),
                DeclStmt(  # 4
                    specifiers=[
                        RecordDecl(struct_or_union="struct", name="B", type=struct_type)
                    ],
                    declarators=[TypeOrVarDecl(name="x", type=struct_type)],
                ),
                DeclStmt(  # 5
                    specifiers=[
                        EnumDecl(
                            name="C",
                            enumerators=[
                                Enumerator(name="D", enum_type=enum_type),
                                Enumerator(name="B", enum_type=enum_type),
                                Enumerator(name="C", enum_type=enum_type),
                            ],
                        )
                    ],
                    declarators=[TypeOrVarDecl(name="t", type=enum_type)],
                ),
                DeclStmt(  # 6
                    specifiers=[EnumDecl(name="C")],
                    declarators=[TypeOrVarDecl(name="u", type=enum_type)],
                ),
            ],
        ),
    )


def test_funcdef():
    parser = get_parser("funcdef.txt")
    parser.nexttoken()
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))

    check_ast(
        ast,
        TranslationUnit(
            body=[
                FunctionDef(
                    func_name="main",
                    func_type=FunctionType(
                        [
                            IntType(),
                            ArrayPtrType(ArrayType(PointerType(CharType()), None)),
                        ],
                        IntType(),
                    ),
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="f",
                            type=FunctionType(
                                [
                                    PointerType(
                                        FunctionType(
                                            [IntType()],
                                            IntType(),
                                        )
                                    )
                                ],
                                IntType(),
                            ),
                        ),
                        TypeOrVarDecl(name="a", type=IntType()),
                    ]
                ),
                DeclStmt(
                    declarators=[
                        TypeOrVarDecl(
                            name="printf",
                            type=FunctionType(
                                [PointerType(CharType())],
                                VoidType(),
                                has_varparam=True,
                            ),
                        )
                    ]
                ),
            ]
        ),
    )
