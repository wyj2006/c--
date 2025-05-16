from parse.memory import memorize, memorize_left_rec
from parse.parser_base import ParserBase
from parse.parser import Parser


def generic_syntax_error(parser: ParserBase):
    """通用语法错误"""
    from basic import Error

    return Error("语法错误", parser.last_token.location)
