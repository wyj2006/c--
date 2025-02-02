from typing import Callable
from AST.Expr import (
    BinaryOperator,
    UnaryOperator,
    IntegerLiteral,
    FloatLiteral,
    CharLiteral,
    StringLiteral,
)
from AST.Node import Node


class Visitor:
    """遍历语法树节点"""

    def generic_visit(
        self, node: Node, callback: Callable[[Node, "Visitor"], None] = None
    ):
        """
        通用访问方法
        """
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
                    callback(i, self)
            else:
                callback(child, self)

    def visit_Node(self, node: Node):
        self.generic_visit(node)


class DumpVisitor(Visitor):
    """输出语法树"""

    def visit_Node(self, node: Node, indent=0):
        print(" " * 2 * indent + node.__class__.__name__, end=" ")
        for i in node._attributes:
            if hasattr(node, i):
                print(getattr(node, i), end=" ")
            else:
                print(end="")
        print()
        self.generic_visit(node, lambda node, _: node.accept(self, indent + 1))

    def visit_BinaryOperator(self, node: BinaryOperator, indent=0):
        print(" " * 2 * indent + node.__class__.__name__, end=" ")
        print(node.op.value)
        self.generic_visit(node, lambda node, _: node.accept(self, indent + 1))

    def visit_UnaryOperator(self, node: UnaryOperator, indent=0):
        print(" " * 2 * indent + node.__class__.__name__, end=" ")
        print(node.op.value)
        self.generic_visit(node, lambda node, _: node.accept(self, indent + 1))


class FormatVisitor(Visitor):
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
        return node.value

    def visit_FloatLiteral(self, node: FloatLiteral):
        return node.value

    def visit_CharLiteral(self, node: CharLiteral):
        return node.value

    def visit_StringLiteral(self, node: StringLiteral):
        return node.value

    def visit_BinaryOperator(self, node: BinaryOperator):
        # TODO: 优先级
        return node.left.accept(self) + node.op.value + node.right.accept(self)

    def visit_UnaryOperator(self, node: UnaryOperator):
        # TODO: 优先级
        return node.op.value + node.operand.accept(self)

    # FIXME:
