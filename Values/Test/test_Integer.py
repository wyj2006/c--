from Test.Common import *


def test_init():
    t = UIntType()
    assert UInt(0) == 0
    assert UInt(-1) == t.range[1]
    assert UInt(t.range[1]) == t.range[1]
    assert UInt(t.range[1] + 1) == 0
    assert UInt(t.range[1] * 2) == t.range[1] - 1
    assert UInt(t.range[1] * 2 + 1) == t.range[1]
    assert UInt(-2) == t.range[1] - 1
    assert UInt(-t.range[1]) == 1

    t = IntType()
    assert Int(0) == 0
    assert Int(-1) == -1
    assert Int(t.range[1]) == t.range[1]
    assert Int(t.range[1] + 1) == t.range[0]
    assert Int(t.range[1] * 2) == t.range[0] + t.range[1] - 1
    assert Int(t.range[1] * 2 + 1) == t.range[0] + t.range[1]
    assert Int(-t.range[1]) == -t.range[1]
