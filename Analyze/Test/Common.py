from Parse.Test.Common import *
from Analyze import *


def check_type(actual: Type, expected: Type):
    assert type(actual) == type(expected)
    for key, val in expected.__dict__.items():
        assert hasattr(actual, key)
        attr = getattr(actual, key)
        if isinstance(attr, Type):
            check_type(attr, val)
        elif isinstance(attr, Node):
            check_ast(attr, val)
        elif isinstance(attr, list):
            for i, v in enumerate(attr):
                if isinstance(v, Type):
                    check_type(v, val[i])
                elif isinstance(v, Node):
                    check_ast(v, val[i])
        else:
            assert attr == val


def check_ast(node: Node, expected: Node):
    assert type(node) == type(expected)
    for key, val in expected.__dict__.items():
        assert hasattr(node, key)
        attr = getattr(node, key)
        if isinstance(attr, Node):
            check_ast(attr, val)
        elif isinstance(attr, Type):
            check_type(attr, val)
        elif isinstance(attr, list):
            for i, v in enumerate(attr):
                if isinstance(v, Node):
                    check_ast(v, val[i])
                elif isinstance(v, Type):
                    check_type(v, val[i])
        else:
            assert attr == val
