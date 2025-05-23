from typing import Callable
from cast import (
    Node,
    IntegerLiteral,
    FloatLiteral,
    CharLiteral,
    StringLiteral,
    Reference,
    Declaration,
    NameDeclarator,
    FunctionDeclarator,
    ArrayDeclarator,
    TypedefSpecifier,
    BinaryOperator,
    UnaryOperator,
    PointerDeclarator,
    BasicTypeSpecifier,
    BitIntSpecifier,
    EnumDecl,
    RecordDecl,
    TypeQualifier,
    AtomicSpecifier,
    TypeOfSpecifier,
)
from .visitor import Visitor


class UnParseVisitor(Visitor):
    """将语法树输出为源代码"""

    def generic_visit(
        self, node: Node, callback: Callable[[Node, "Visitor"], None] = None
    ):
        """
        通用访问方法
        """
        code = ""
        if callback == None:
            callback = lambda node, visitor: node.accept(visitor)
        for field in node._fields:
            child = getattr(node, field, None)
            if child == None:
                continue
            if isinstance(child, (list, tuple)):
                for i in child:
                    if i == None:
                        continue
                    code += callback(i, self)
            else:
                code += callback(child, self)
        return code

    def visit_IntegerLiteral(self, node: IntegerLiteral):
        return str(node.value)

    def visit_FloatLiteral(self, node: FloatLiteral):
        return str(node.value)

    def visit_CharLiteral(self, node: CharLiteral):
        return str(node.value)

    def visit_StringLiteral(self, node: StringLiteral):
        return str(node.value)

    def visit_BinaryOperator(self, node: BinaryOperator):
        # TODO: 优先级
        return f"{node.left.accept(self)}{node.op.value}{node.right.accept(self)}"

    def visit_UnaryOperator(self, node: UnaryOperator):
        # TODO: 优先级
        return f"{node.op.value}{node.operand.accept(self)}"

    def visit_Reference(self, node: Reference):
        return node.name

    def visit_Declaration(self, node: Declaration):
        l = []
        for i in node.specifiers:
            a = i.accept(self)
            assert a != None, i
            l.append(a)
        code = " ".join(l).strip()
        l = []
        for i in node.declarators:
            a = i.accept(self)
            assert a != None, i
            l.append(a)
        return code + " ".join(l)

    def visit_NameDeclarator(self, node: NameDeclarator):
        return node.name

    def visit_PointerDeclarator(self, node: PointerDeclarator, parent: Node = None):
        code: str = "*" + " ".join([i.accept(self) for i in node.qualifiers])

        if isinstance(parent, (FunctionDeclarator, ArrayDeclarator)):
            code = "(" + code + ")"

        if node.declarator != None:
            code += node.declarator.accept(self)
        return code.strip()

    def visit_ArrayDeclarator(self, node: ArrayDeclarator, parent: Node = None):
        if node.declarator != None:
            code: str = node.declarator.accept(self, node)
        else:
            code = ""

        code += "["

        if node.is_static:
            code += "static "

        if node.is_star_modified:
            code += "* "

        for i in node.qualifiers:
            code += f"{i.accept(self)} "

        if node.size != None:
            code += f"{node.size.accept(self)}"

        code = code.strip()
        code += "]"
        return code.strip()

    def visit_FunctionDeclarator(self, node: FunctionDeclarator, parent: Node = None):
        if node.declarator != None:
            code: str = node.declarator.accept(self, node)
        else:
            code = ""

        code += "("

        for param in node.parameters:
            code += f"{param.accept(self)},"

        if node.has_varparam:
            code += "..."

        code = code.strip(",")
        code += ")"
        return code.strip()

    def visit_BasicTypeSpecifier(self, node: BasicTypeSpecifier):
        return node.specifier_name

    def visit_BitIntSpecifier(self, node: BitIntSpecifier):
        return f"_BitInt({node.size.accept(self)})"

    def visit_RecordDecl(self, node: RecordDecl):
        return f"{node.struct_or_union} {node.name}"

    def visit_EnumDecl(self, node: EnumDecl):
        return f"enum {node.name}"

    def visit_AtomicSpecifier(self, node: AtomicSpecifier):
        return f"_Atomic({node.type_name.accept(self)})"

    def visit_TypedefSpecifier(self, node: TypedefSpecifier):
        return node.specifier_name

    def visit_TypeQualifier(self, node: TypeQualifier):
        return node.qualifier.value

    def visit_TypeOfSpecifier(self, node: TypeOfSpecifier):
        code = "typeof"
        if node.is_unqual:
            code += "_unqual"
        code += "("
        code += node.arg.accept(self)
        code += ")"
        return code

    # FIXME:
