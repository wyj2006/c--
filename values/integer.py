from typing import Any, Callable
from typesystem import (
    IntegerType,
    ShortType,
    UShortType,
    LongType,
    ULongLongType,
    UIntType,
    ULongType,
    LongLongType,
    IntType,
)
from values.value import Value


class Integer(Value):
    type: IntegerType

    def __init__(self, value: int, type: IntegerType):
        super().__init__(type)
        self._value = int(value)
        self.adjust()

    @property
    def value(self):
        return self._value

    def adjust(self):
        l, r = self.type.range
        while self._value > r:
            self._value = l + ((self._value - r) - 1) % (r - l + 1)
        while self._value < l:
            self._value = r - ((l - self._value) - 1) % (r - l + 1)

    def __repr__(self):
        return f"{self.__class__.__name__}({self.value})"

    def __str__(self):
        return f"{self.value}"

    def generic_bin_op(
        self,
        other,
        op: Callable[[int, int], Any],
        result_map: Callable[[int, IntegerType], Any] = None,
    ):
        type = self.type
        if isinstance(other, Integer):
            other_value = other.value
            if other.type.size > self.type.size:
                type = other.type
        elif isinstance(other, int):
            other_value = other
        elif isinstance(other, float):
            other_value = int(other)
        else:
            return NotImplemented
        if result_map == None:
            result_map = lambda v, t: Integer(v, t)
        return result_map(op(self.value, other_value), type)

    def generic_unary_op(
        self,
        op: Callable[[int], Any],
        result_map: Callable[[int, IntegerType], Any] = None,
    ):
        if result_map == None:
            result_map = lambda v, t: Integer(v, t)
        return result_map(op(self.value), self.type)

    def __float__(self) -> float:
        return float(self.value)

    def __int__(self) -> int:
        return self.value

    def __abs__(self):
        return Integer(abs(self.value), self.type)

    def __bool__(self) -> bool:
        return bool(self.value)


class Int(Integer):
    def __init__(self, value):
        super().__init__(value, IntType())


class UInt(Integer):
    def __init__(self, value):
        super().__init__(value, UIntType())


class Short(Integer):
    def __init__(self, value):
        super().__init__(value, ShortType())


class UShort(Integer):
    def __init__(self, value):
        super().__init__(value, UShortType())


class Long(Integer):
    def __init__(self, value):
        super().__init__(value, LongType())


class ULong(Integer):
    def __init__(self, value):
        super().__init__(value, ULongType())


class LongLong(Integer):
    def __init__(self, value):
        super().__init__(value, LongLongType())


class ULongLong(Integer):
    def __init__(self, value):
        super().__init__(value, ULongLongType())
