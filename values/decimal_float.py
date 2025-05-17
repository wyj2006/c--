from typing import Any, Callable
from values.bin_float import BinFloat
from values.integer import Integer
from values.value import Value
from typesystem import DecimalType
from decimal import Decimal


class DecimalFloat(Value):
    """十进制浮点数"""

    def __init__(self, value, type: DecimalType):
        super().__init__(type)
        self._value = Decimal(value)

    @property
    def value(self):
        return self._value

    def __str__(self):
        return f"{self.value}"

    def generic_bin_op(
        self,
        other,
        op: Callable[[Decimal, Decimal], Any],
        result_map: Callable[[Decimal, DecimalType], Any] = None,
    ):
        type = self.type
        if isinstance(other, DecimalFloat):
            other_value = other.value
            if other.type.size > self.type.size:
                type = self.type.size
        elif isinstance(other, BinFloat):
            other_value = Decimal(other.value)
        elif isinstance(other, Integer):
            other_value = Decimal(other.value)
        elif isinstance(other, int):
            other_value = Decimal(other)
        elif isinstance(other, float):
            other_value = Decimal(other)
        else:
            return NotImplemented
        if result_map == None:
            result_map = lambda v, t: DecimalFloat(v, t)
        return result_map(op(self.value, other_value), type)

    def generic_unary_op(
        self,
        op: Callable[[Decimal], Any],
        result_map: Callable[[Decimal, DecimalType], Any] = None,
    ):
        if result_map == None:
            result_map = lambda v, t: DecimalFloat(v, t)
        return result_map(op(self.value), self.type)
