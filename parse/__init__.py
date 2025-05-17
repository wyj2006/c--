from .memory import memorize, memorize_left_rec
from .parser_base import ParserBase
from .parser import Parser


def generic_syntax_error(parser: ParserBase):
    """通用语法错误"""
    from basic import Error

    return Error("语法错误", parser.last_token.location)
