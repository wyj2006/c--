import os
from basic import (
    Visitor,
    IntegerLiteral,
    Expr,
    BinOpKind,
    UnaryOpKind,
    BinaryOperator,
    UnaryOperator,
    Defined,
    HasCAttribute,
    HasEmbed,
    HasInclude,
    Reference,
    Embed,
)
from basic import Error, FileReader, Symtab
from lex.macro import Macro
from lex.preprocessor import Preprocessor


class Evaluater(Visitor):
    """常量表达式求值, 主要用于#if和#elif"""

    operator = {
        BinOpKind.ADD: lambda a, b: a + b,
        BinOpKind.SUB: lambda a, b: a - b,
        BinOpKind.MUL: lambda a, b: a * b,
        BinOpKind.DIV: lambda a, b: a / b,
        BinOpKind.MOD: lambda a, b: a % b,
        BinOpKind.LSHIFT: lambda a, b: a << b,
        BinOpKind.RSHIFT: lambda a, b: a >> b,
        BinOpKind.EQ: lambda a, b: a == b,
        BinOpKind.NEQ: lambda a, b: a != b,
        BinOpKind.GT: lambda a, b: a > b,
        BinOpKind.GTE: lambda a, b: a >= b,
        BinOpKind.LT: lambda a, b: a < b,
        BinOpKind.LTE: lambda a, b: a <= b,
        BinOpKind.BITAND: lambda a, b: a & b,
        BinOpKind.BITOR: lambda a, b: a | b,
        BinOpKind.BITXOR: lambda a, b: a ^ b,
        BinOpKind.AND: lambda a, b: a and b,
        BinOpKind.OR: lambda a, b: a or b,
        UnaryOpKind.POSITIVE: lambda a: +a,
        UnaryOpKind.NEGATIVE: lambda a: -a,
        UnaryOpKind.INVERT: lambda a: ~a,
        UnaryOpKind.NOT: lambda a: not a,
    }

    def __init__(self, symtab: Symtab = None):
        super().__init__()
        if symtab == None:
            symtab = Symtab()
        self.symtab = symtab

    def visit_Expr(self, node: Expr):
        raise Error(f"{node.__class__.__name__}不应该出现在这个上下文中", node.location)

    def visit_IntegerLiteral(self, node: IntegerLiteral):
        from analyses import ConstEvaluater

        node.value = node.accept(ConstEvaluater(self.symtab)).value

    def visit_BinaryOperator(self, node: BinaryOperator):
        if node.op not in self.operator:
            raise Error("这个上下文不允许该运算", node.location)
        node.left.accept(self)
        node.right.accept(self)
        node.value = self.operator[node.op](node.left.value, node.right.value)

    def visit_UnaryOperator(self, node: UnaryOperator):
        if node.op not in self.operator:
            raise Error("这个上下文不允许该运算", node.location)
        node.operand.accept(self)
        node.value = self.operator[node.op](node.operand.value)

    def visit_Defined(self, node: Defined):
        if self.symtab.lookup(node.name) != None or node.name in (
            "__has_include",
            "__has_embed",
            "__has_c_attribute",
        ):
            node.value = 1
        else:
            node.value = 0

    def visit_Reference(self, node: Reference):
        # 如果node.name in self.macros, 那么它应该已经被替换了
        node.value = 0

    def visit_HasCAttribute(self, node: HasCAttribute):
        attribute = node.attribute
        attribute_names = self.symtab.attribute_names
        if (
            attribute.prefix_name not in attribute_names
            or attribute.name not in attribute_names[attribute.prefix_name]
        ):
            node.value = 0
        else:
            node.value = 1

    def visit_HasInclude(self, node: HasInclude):
        filepath = Preprocessor.find_include_file(
            node.filename,
            node.search_current_path,
            os.path.dirname(node.location[0]["filename"]),
        )
        if filepath == None:
            node.value = 0
        else:
            node.value = 1

    def visit_HasEmbed(self, node: HasEmbed):
        support_parameters = True  # 参数列表中的参数都被编译器支持
        try:
            Embed(parameters=node.parameters).analyze_parameters()
        except Error:
            support_parameters = False

        filepath = Preprocessor.find_include_file(
            node.filename,
            node.search_current_path,
            os.path.dirname(node.location[0]["filename"]),
        )
        file_found = filepath != None  # 文件存在
        file_is_empty = False  # 文件为空
        if filepath != None:
            reader = FileReader(filepath, "rb")
            file_is_empty = len(reader.lines) > 0

        if file_found and not file_is_empty and support_parameters:
            node.value = 0  # __STDC_EMBED_FOUND__
        elif file_found and file_is_empty and support_parameters:
            node.value = 1  # __STDC_EMBED_EMPTY__
        else:
            node.value = 2  # __STDC_EMBED_NOT_FOUND__
