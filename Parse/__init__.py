from Parse.Memory import memorize, memorize_left_rec
from Parse.ParserBase import ParserBase
from Parse.Parser import Parser


def generic_syntax_error(parser: ParserBase):
    """通用语法错误"""
    from Basic import Error

    return Error("语法错误", parser.last_token.location)
