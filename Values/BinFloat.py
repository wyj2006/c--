from typing import Any, Callable
from Values.Value import Value
from Values.Integer import Integer
from Types import FloatType, DoubleType, LongDoubleType, BinaryFloatType


class BinFloat(Value):
    """二进制浮点数"""

    def __init__(self, value: float, type: BinaryFloatType):
        super().__init__(type)
        self._value = float(value)

    @property
    def value(self):
        return self._value

    def __repr__(self):
        return f"{self.__class__.__name__}({self.value})"

    def __str__(self):
        return f"{self.value}"

    def generic_bin_op(
        self,
        other,
        op: Callable[[float, float], Any],
        result_map: Callable[[float, BinaryFloatType], Any] = None,
    ):
        type = self.type
        if isinstance(other, BinFloat):
            other_value = other.value
            if other.type.size > self.type.size:
                type = other.type
        elif isinstance(other, Integer):
            other_value = other.value
        elif isinstance(other, int):
            other_value = float(other)
        elif isinstance(other, float):
            other_value = other
        else:
            return NotImplemented
        if result_map == None:
            result_map = lambda v, t: BinFloat(v, t)
        return result_map(op(self.value, other_value), type)

    def generic_unary_op(
        self,
        op: Callable[[float], Any],
        result_map: Callable[[float, BinaryFloatType], Any] = None,
    ):
        if result_map == None:
            result_map = lambda v, t: BinFloat(v, t)
        return result_map(op(self.value), self.type)

    def __float__(self) -> float:
        return float(self.value)

    def __int__(self) -> int:
        return int(self.value)

    def __abs__(self):
        return BinFloat(abs(self.value), self.type)

    def __bool__(self) -> bool:
        return bool(self.value)


class Float(BinFloat):
    def __init__(self, value):
        super().__init__(value, FloatType())


class Double(BinFloat):
    def __init__(self, value):
        super().__init__(value, DoubleType())


class LongDouble(BinFloat):
    def __init__(self, value):
        super().__init__(value, LongDoubleType())
