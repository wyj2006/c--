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

    def __call__(self, imag=0):
        from values import Imaginary

        return Imaginary(imag, self)

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name=f"{self.imag_type} _Imaginary")
        )


class FloatImaginaryType(ImaginaryType):
    def __init__(self, attribute_specifiers=None):
        super().__init__(FloatType(), attribute_specifiers)


class DoubleImaginaryType(ImaginaryType):
    def __init__(self, attribute_specifiers=None):
        super().__init__(DoubleType(), attribute_specifiers)


class LongDoubleImaginaryType(ImaginaryType):
    def __init__(self, attribute_specifiers=None):
        super().__init__(LongDoubleType(), attribute_specifiers)
