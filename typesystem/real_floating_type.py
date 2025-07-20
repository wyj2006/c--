import ctypes
from .floating_type import FloatingType
from .integer_type import BitIntType


class RealFloatingType(FloatingType):
    is_real_type = True


class BinaryFloatType(RealFloatingType):
    def __call__(self, value=0):
        from values import BinFloat

        return BinFloat(value, self)

    def __eq__(self, other):
        return isinstance(other, BinaryFloatType) and self.size == other.size


class FloatType(BinaryFloatType):
    size = 4
    alignment = 4

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="float"))


class DoubleType(BinaryFloatType):
    size = 8
    alignment = 8

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="double"))


class LongDoubleType(BinaryFloatType):
    size = 16
    alignment = ctypes.alignment(ctypes.c_longdouble)

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="long double"))


class DecimalType(RealFloatingType):
    exp_size: int  # 指数位大小
    man_size: int  # 尾数位大小

    def __init__(self, attribute_specifiers=None):
        super().__init__(attribute_specifiers)
        self.exp_type = BitIntType(self.exp_size)  # 指数位
        self.man_type = BitIntType(self.man_size)  # 尾数位

    def __eq__(self, other):
        return (
            isinstance(other, DecimalType)
            and self.exp_size == other.exp_size
            and self.man_size == other.man_size
        )


class Decimal32Type(DecimalType):
    size = 4
    exp_size = 7
    man_size = 24
    alignment = 4

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="Decimal32"))


class Decimal64Type(DecimalType):
    size = 8
    exp_size = 10
    man_size = 53
    alignment = 8

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="Decimal64"))


class Decimal128Type(DecimalType):
    size = 16
    exp_size = 15
    man_size = 112
    alignment = 16

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="Decimal128"))
