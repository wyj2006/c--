from .floating_type import FloatingType
from .real_floating_type import (
    DoubleType,
    FloatType,
    LongDoubleType,
    RealFloatingType,
)


class ImaginaryType(FloatingType):
    def __init__(self, imag_type: RealFloatingType, attribute_specifiers=None):
        super().__init__(attribute_specifiers)
        self.imag_type = imag_type


class FloatImaginaryType(ImaginaryType):
    def __init__(self, attribute_specifiers=None):
        super().__init__(FloatType(), attribute_specifiers)

    def genDeclaration(self, declaration):
        from basic import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="float _Imaginary")
        )


class DoubleImaginaryType(ImaginaryType):
    def __init__(self, attribute_specifiers=None):
        super().__init__(DoubleType(), attribute_specifiers)

    def genDeclaration(self, declaration):
        from basic import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="double _Imaginary")
        )


class LongDoubleImaginaryType(ImaginaryType):
    def __init__(self, attribute_specifiers=None):
        super().__init__(LongDoubleType(), attribute_specifiers)

    def genDeclaration(self, declaration):
        from basic import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="long double _Imaginary")
        )
