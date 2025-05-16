from test_common import *


def test_attribute():
    parser = get_parser("attribute.txt")
    parser.nexttoken()
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(AttrAnalyzer(symtab))

    check_ast(
        ast,
        TranslationUnit(
            body=[
                AttributeDeclStmt(
                    attribute_specifiers=[
                        AttributeSpecifier(
                            attributes=[
                                DeprecatedAttr(),
                                Attribute(prefix_name="hal", name="daisy"),
                            ]
                        )
                    ]
                )
            ]
        ),
    )
