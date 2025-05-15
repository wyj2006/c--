from typing import Any, Callable
from Types import Type


class Value:
    def __init__(self, type: Type):
        self.type = type

    def generic_bin_op(
        self,
        other,
        op: Callable[[Any, Any], Any],
        result_map: Callable[[Any, Type], Any] = None,
    ):
        return NotImplemented

    def generic_unary_op(
        self,
        op: Callable[[Any], Any],
        result_map: Callable[[Any, Type], Any] = None,
    ):
        return NotImplemented

    def __add__(self, other):
        return self.generic_bin_op(other, lambda a, b: a + b)

    def __sub__(self, other):
        return self.generic_bin_op(other, lambda a, b: a - b)

    def __mul__(self, other):
        return self.generic_bin_op(other, lambda a, b: a * b)

    def __floordiv__(self, other):
        return self.generic_bin_op(other, lambda a, b: a // b)

    def __truediv__(self, other) -> float:
        return self.generic_bin_op(other, lambda a, b: a / b, lambda a, _: a)

    def __mod__(self, other):
        return self.generic_bin_op(other, lambda a, b: a % b)

    def __pow__(self, other):
        return self.generic_bin_op(other, lambda a, b: a**b)

    def __and__(self, other):
        return self.generic_bin_op(other, lambda a, b: a & b)

    def __or__(self, other):
        return self.generic_bin_op(other, lambda a, b: a | b)

    def __xor__(self, other):
        return self.generic_bin_op(other, lambda a, b: a ^ b)

    def __lshift__(self, other):
        return self.generic_bin_op(other, lambda a, b: a << b)

    def __rshift__(self, other):
        return self.generic_bin_op(other, lambda a, b: a >> b)

    def __radd__(self, other):
        return self.generic_bin_op(other, lambda a, b: b + a)

    def __rsub__(self, other):
        return self.generic_bin_op(other, lambda a, b: b - a)

    def __rmul__(self, other):
        return self.generic_bin_op(other, lambda a, b: b * a)

    def __rfloordiv__(self, other):
        return self.generic_bin_op(other, lambda a, b: b // a)

    def __rtruediv__(self, other) -> float:
        return self.generic_bin_op(other, lambda a, b: a / b, lambda a, _: a)

    def __rmod__(self, other):
        return self.generic_bin_op(other, lambda a, b: b % a)

    def __rpow__(self, other):
        return self.generic_bin_op(other, lambda a, b: b**a)

    def __rand__(self, other):
        return self.generic_bin_op(other, lambda a, b: b & a)

    def __ror__(self, other):
        return self.generic_bin_op(other, lambda a, b: b | a)

    def __rxor__(self, other):
        return self.generic_bin_op(other, lambda a, b: b ^ a)

    def __rlshift__(self, other):
        return self.generic_bin_op(other, lambda a, b: b << a)

    def __rrshift__(self, other):
        return self.generic_bin_op(other, lambda a, b: b >> a)

    def __neg__(self):
        return self.generic_unary_op(lambda a: -a)

    def __pos__(self):
        return self.generic_unary_op(lambda a: +a)

    def __invert__(self):
        return self.generic_unary_op(lambda a: ~a)

    def __eq__(self, other):
        return self.generic_bin_op(other, lambda a, b: a == b)

    def __ne__(self, other):
        return self.generic_bin_op(other, lambda a, b: a != b, lambda a, _: a)

    def __lt__(self, other):
        return self.generic_bin_op(other, lambda a, b: a < b, lambda a, _: a)

    def __le__(self, other):
        return self.generic_bin_op(other, lambda a, b: a <= b, lambda a, _: a)

    def __gt__(self, other):
        return self.generic_bin_op(other, lambda a, b: a > b, lambda a, _: a)

    def __ge__(self, other):
        return self.generic_bin_op(other, lambda a, b: a >= b, lambda a, _: a)
