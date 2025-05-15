import ctypes
from typing import TYPE_CHECKING, Union
from Types.Type import Type

if TYPE_CHECKING:
    from AST import Expr


class IntegerType(Type):
    """整数类型"""

    is_integer_type = True
    is_real_type = True
    is_arithmetic_type = True
    is_scalar_type = True

    size: int
    signed: bool  # 有无符号

    def __eq__(self, other):
        return (
            isinstance(other, IntegerType)
            and self.size == other.size
            and self.signed == other.signed
        )

    @property
    def range(self):
        if self.signed:
            return -(2 ** (self.size * 8 - 1)), 2 ** (self.size * 8 - 1) - 1
        return 0, 2 ** (self.size * 8) - 1

    def __call__(self, value):
        """创建类型对应的值"""
        from Values import Integer

        return Integer(value, self)


class ShortType(IntegerType):
    size = ctypes.sizeof(ctypes.c_short)
    signed = True

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="short"))


class IntType(IntegerType):
    size = ctypes.sizeof(ctypes.c_int)
    signed = True

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="int"))


class LongType(IntegerType):
    size = ctypes.sizeof(ctypes.c_long)
    signed = True

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="long"))


class LongLongType(IntegerType):
    size = ctypes.sizeof(ctypes.c_longlong)
    signed = True

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="long long"))


class UShortType(IntegerType):
    size = ctypes.sizeof(ctypes.c_ushort)
    signed = False

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="unsigned short")
        )


class UIntType(IntegerType):
    size = ctypes.sizeof(ctypes.c_uint)
    signed = False

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="unsigned int"))


class ULongType(IntegerType):
    size = ctypes.sizeof(ctypes.c_ulong)
    signed = False

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="unsigned long")
        )


class ULongLongType(IntegerType):
    size = ctypes.sizeof(ctypes.c_ulonglong)
    signed = False

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="unsigned long long")
        )


class BitIntType(IntegerType):
    def __init__(
        self, size: Union["Expr", int], signed: bool = True, attribute_specifiers=None
    ):
        from AST import Expr, IntegerLiteral

        super().__init__(attribute_specifiers)
        self.signed = signed
        self.size_expr = None
        self._size = None
        if isinstance(size, Expr):
            self.size_expr = size
            self._size = None
        else:
            self._size = size
            self.size_expr = IntegerLiteral(value=size)

    @property
    def size(self):
        if self._size != None:
            return (self._size - 1) // 8
        if hasattr(self.size_expr, "value"):
            return (self.size_expr.value - 1) // 8
        return None

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier, BitIntSpecifier

        if not self.signed:
            declaration.specifiers.append(BasicTypeSpecifier(specifier_name="unsigned"))
        declaration.specifiers.append(BitIntSpecifier(size=self.size_expr))
