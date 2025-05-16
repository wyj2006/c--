from typing import Any, Callable
from values.bin_float import BinFloat
from values.decimal_float import DecimalFloat
from values.integer import Integer
from values.value import Value
from basic import ComplexType


class Complex(Value):
    def __init__(self, real, imag, type: ComplexType):
        super().__init__(type)
        self._value = complex(real, imag)

    @property
    def value(self):
        return self._value

    def __str__(self):
        return f"{self.value}"

    def generic_bin_op(
        self,
        other,
        op: Callable[[complex, complex], Any],
        result_map: Callable[[complex, ComplexType], Any] = None,
    ):
        type = self.type

        if isinstance(other, Complex):
            other_value = other.value
            if other.type.size > self.type.size:
                type = self.type.size
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
        else:
            return NotImplemented
        if result_map == None:
            result_map = lambda v, t: Complex(v, t)
        return result_map(op(self.value, other_value), type)

    def generic_unary_op(
        self,
        op: Callable[[complex], Any],
        result_map: Callable[[complex, ComplexType], Any] = None,
    ):
        if result_map == None:
            result_map = lambda v, t: Complex(v, t)
        return result_map(op(self.value), self.type)
