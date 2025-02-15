from Test.Common import *


def test_attribute():
    parser = get_parser("attribute.txt")
    parser.nexttoken()
    for i in (
        AttributeDeclStmt(
            attribute_specifiers=[
                AttributeSpecifier(
                    attributes=[
                        DeprecatedAttr(),
                        Attribute(prefix_name="hal", name="daisy"),
                    ]
                )
            ]
        ),
    ):
        a = parser.attribute_declaration()
        a.accept(DumpVisitor())
        check_ast(a, i)
    assert parser.curtoken().kind == TokenKind.END
