"""与类型转换有关的操作"""

from typing import TYPE_CHECKING, Callable, TypeVar
from Basic import (
    Function,
    EnumConst,
)
from AST import (
    ImplicitCast,
    Expr,
    Node,
    StringLiteral,
    UnaryOperator,
    UnaryOpKind,
    BinaryOperator,
    BinOpKind,
    TypeOfSpecifier,
    TypeOrVarDecl,
    MemberRef,
    CompoundLiteral,
    Reference,
    ArraySubscript,
)
from Types import (
    Type,
    is_compatible_type,
    ArrayType,
    PointerType,
    FunctionType,
    remove_atomic,
    remove_qualifier,
    integer_promotion,
    QualifiedType,
    RecordType,
)

if TYPE_CHECKING:
    from Analyze.TypeChecker import TypeChecker

_T = TypeVar("_T")


def is_lvalue(node: Expr):
    """判断node是否是左值表达式"""
    if hasattr(node, "is_lvalue"):
        return node.is_lvalue

    if isinstance(node, Reference):
        if isinstance(node.symbol, (Function, EnumConst)):
            return False
        return True
    if isinstance(node, (StringLiteral, CompoundLiteral)):
        return True
    if isinstance(node, MemberRef):
        return (
            node.is_arrow == False and is_lvalue(node.target) or node.is_arrow == True
        )
    if isinstance(node, UnaryOperator) and node.op == UnaryOpKind.DEREFERENCE:
        return True
    if isinstance(node, ArraySubscript):
        return True
    return False


def is_modifiable_lvalue(node: Expr):
    """判断是否是可修改左值"""
    if isinstance(node.type, QualifiedType) and node.type.has_const():
        return False
    if isinstance(node.type, ArrayType):
        return False
    if not node.type.is_complete():
        return False
    if isinstance(node.type, RecordType):
        for name, member in node.type.members.items():
            if isinstance(member.type, QualifiedType) and member.type.has_const():
                return False
    return True


def implicit_cast(node: Expr, type: Type):
    """将node的类型隐式转换为type, 并返回转换后的节点"""
    if hasattr(node, "type") and is_compatible_type(node.type, type):
        return node
    return ImplicitCast(
        type=type,
        expr=node,
        location=node.location,
    )


def generic_implicit_cast(func: Callable[["TypeChecker", Node], _T]):
    """
    对访问函数的返回值进行通用的隐式转换
    同时会维护TypeChecker.path
    """

    def wrapper(self: "TypeChecker", node: Node) -> _T:
        ret = node
        if not hasattr(node, "value"):
            # 在ConstEvaluater中处理过了
            self.path.append(node)
            ret = func(self, node)
            self.path.pop()

        if not isinstance(node, Expr):
            return ret
        if not hasattr(node, "type"):
            return ret

        if isinstance(node.type, ArrayType):  # 数组到指针转换
            if self.path and (
                (
                    isinstance(self.path[-1], UnaryOperator)
                    and (
                        self.path[-1].op
                        in (
                            UnaryOpKind.ADDRESS,  # 作为取址运算符的操作数
                            UnaryOpKind.SIZEOF,  # 作为 sizeof 的操作数
                        )
                    )
                )
                or isinstance(  # 作为 typeof 和 typeof_unqual 的操作数
                    self.path[-1], TypeOfSpecifier
                )
                or (  # 作为用于数组初始化的字符串字面量
                    isinstance(self.path[-1], TypeOrVarDecl)
                    and isinstance(node, StringLiteral)
                )
            ):
                return ret
            return implicit_cast(node, PointerType(node.type.element_type))

        if isinstance(node.type, FunctionType):  # 函数到指针转换
            if self.path and (
                (
                    isinstance(self.path[-1], UnaryOperator)
                    and (
                        self.path[-1].op
                        in (
                            UnaryOpKind.ADDRESS,  # 作为取址运算符的操作数
                            UnaryOpKind.SIZEOF,  # 作为 sizeof 的操作数
                        )
                    )
                )
                or isinstance(  # 作为 typeof 和 typeof_unqual 的操作数
                    self.path[-1], TypeOfSpecifier
                )
            ):
                return ret
            return implicit_cast(node, PointerType(node.type))

        if is_lvalue(node) and not isinstance(node.type, ArrayType):  # 左值转换
            if self.path and (
                (
                    isinstance(self.path[-1], UnaryOperator)
                    and (
                        self.path[-1].op
                        in (
                            UnaryOpKind.ADDRESS,  # 作为取址运算符的操作数
                            UnaryOpKind.SIZEOF,  # 作为 sizeof 的操作数
                            # 作为前/后自增减运算符的操作数
                            UnaryOpKind.POSTFIX_DEC,
                            UnaryOpKind.PREFIX_DEC,
                            UnaryOpKind.POSTFIX_INC,
                            UnaryOpKind.PREFIX_INC,
                        )
                    )
                )
                or (  # 作为赋值与复合赋值运算符的左操作数
                    isinstance(self.path[-1], BinaryOperator)
                    and self.path[-1].op == BinOpKind.ASSIGN
                    and self.path[-1].left == node
                )
                or (  # 作为成员访问（点）运算符的左操作数
                    isinstance(self.path[-1], MemberRef)
                    and self.path[-1].target == node
                    and self.path[-1].is_arrow == False
                )
            ):
                return ret
            return implicit_cast(node, remove_atomic(remove_qualifier(node.type)))

        return ret

    return wrapper


def integer_promotion_cast(node, type):
    """整数提升转换"""
    implicit_cast(node, integer_promotion(type))


def usual_arithmetic_cast(left: Expr, right: Expr):
    """一般算术转换"""
    # TODO:
    return left, right
