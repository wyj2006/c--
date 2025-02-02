import inspect
import sys
import os

sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(__file__))))

from AST import *
from Basic import *
from Lex import *
from Parse import *


def check_ast(node: Node, expected: Node):
    assert type(node) == type(expected)
    for key, val in expected.__dict__.items():
        assert hasattr(node, key)
        attr = getattr(node, key)
        if isinstance(attr, Node):
            check_ast(attr, val)
        elif isinstance(attr, list):
            for i, v in enumerate(attr):
                if isinstance(v, Node):
                    check_ast(v, val[i])
        else:
            assert attr == val


def get_parser(filename: str):
    caller_frame = inspect.stack()[1]
    caller_file = caller_frame[1]

    reader = FileReader(os.path.join(os.path.dirname(caller_file), filename))
    lexer = Preprocessor(reader)
    parser = Parser(lexer)
    return parser
