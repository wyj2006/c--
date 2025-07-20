from typing import Any, Callable
from values.ccomplex import Complex
from values.decimal_float import DecimalFloat
from values.bin_float import BinFloat
from values.integer import Integer
from typesystem import ImaginaryType, ComplexType


class Imaginary(Complex):
    def __init__(self, imag, type: ImaginaryType):
        super().__init__(0, imag, type)

    def generic_bin_op(
        self,
        other,
        op: Callable[[complex, complex], Any],
        result_map: Callable[[complex, ComplexType], Any] = None,
    ):
        type = ComplexType(self.type.imag_type, self.type.imag_type)

        if isinstance(other, Imaginary):
            other_value = other.value
        elif isinstance(other, Complex):
            other_value = other.value
            type = other.type
        elif isinstance(other, DecimalFloat):
            other_value = complex(other.value)
        elif isinstance(other, BinFloat):
            other_value = complex(other.value)
        elif isinstance(other, Integer):
            other_value = complex(other.value)
        elif isinstance(other, int):
            other_value = complex(other)
        elif isinstance(other, float):
            other_value = complex(other)
        elif isinstance(other, complex):
            other_value = other
        else:
            return NotImplemented
        if result_map == None:
            result_map = lambda v, t: Complex(v.real, v.imag, t)
        return result_map(op(self.value, other_value), type)
