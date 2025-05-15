from Types.FloatingType import FloatingType
from Types.RealFloatingType import (
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


class FloatComplexType(ComplexType):
    size = 8

    def __init__(self, attribute_specifiers=None):
        super().__init__(FloatType(), FloatType(), attribute_specifiers)

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="float _Complex")
        )


class DoubleComplexType(ComplexType):
    size = 16

    def __init__(self, attribute_specifiers=None):
        super().__init__(DoubleType(), DoubleType(), attribute_specifiers)

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="double _Complex")
        )


class LongDoubleComplexType(ComplexType):
    size = 32

    def __init__(self, attribute_specifiers=None):
        super().__init__(LongDoubleType(), LongDoubleType(), attribute_specifiers)

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="long double _Complex")
        )
