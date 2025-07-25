from .floating_type import FloatingType
from .real_floating_type import (
    DoubleType,
    FloatType,
    LongDoubleType,
    RealFloatingType,
)


class ComplexType(FloatingType):

    def __init__(
        self,
        real_type: RealFloatingType,
        imag_type: RealFloatingType,
        attribute_specifiers=None,
    ):
        super().__init__(attribute_specifiers)
        self.real_type = real_type  # 实部类型
        self.imag_type = imag_type  # 虚部类型

    def __call__(self, real=0, imag=0):
        from values import Complex

        return Complex(real, imag, self)

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(
                specifier_name=f"{self.real_type} {self.imag_type if self.real_type!=self.imag_type else ''}_Complex"
            )
        )


class FloatComplexType(ComplexType):
    size = 8

    def __init__(self, attribute_specifiers=None):
        super().__init__(FloatType(), FloatType(), attribute_specifiers)


class DoubleComplexType(ComplexType):
    size = 16

    def __init__(self, attribute_specifiers=None):
        super().__init__(DoubleType(), DoubleType(), attribute_specifiers)


class LongDoubleComplexType(ComplexType):
    size = 32

    def __init__(self, attribute_specifiers=None):
        super().__init__(LongDoubleType(), LongDoubleType(), attribute_specifiers)
