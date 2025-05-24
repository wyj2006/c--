"""
类型的值
值的各个运算分量的类型和结果的类型应该与该值的类型对应的运算函数签名相同
"""

from .value import Value
from .integer import (
    Integer,
    Int,
    Long,
    LongLong,
    ULongLong,
    Short,
    ULong,
    UShort,
    UInt,
)
from .bin_float import BinFloat, Float, Double, LongDouble
from .decimal_float import DecimalFloat
from .imaginary import Imaginary
