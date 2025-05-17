from values.ccomplex import Complex
from typesystem import ImaginaryType


class Imaginary(Complex):
    def __init__(self, imag, type: ImaginaryType):
        super().__init__(0, imag, type)
