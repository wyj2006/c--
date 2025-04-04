from Basic import TokenGen, TokenKind, Token, Error
from .Memory import LR, Head, memorize


class ParserBase:
    def __init__(self, tokengen: TokenGen):
        self.tokengen = tokengen
        self._memo = {}
        self._lr_stack: list[LR] = []
        self._heads: dict[int, Head] = {}

    def save(self):
        return self.tokengen.save()

    def restore(self, *args, **kwargs):
        return self.tokengen.restore(*args, **kwargs)

    def curtoken(self):
        return self.tokengen.curtoken()

    def nexttoken(self):
        return self.tokengen.next()

    def lookahead(self, *args):
        z = self.save()
        for i, v in enumerate(args):
            if self.curtoken().kind != v:
                self.restore(z)
                return False
            if i < len(args) - 1:
                self.nexttoken()  # 防止多读
        self.restore(z)
        return True

    def error(self, msg, location):
        raise Error(msg, location)

    @memorize
    def expect(self, expected: TokenKind, **kwargs):
        """
        判断当前tokenkind是否与期待相等
        如果相等返回当前token, 同时读取下一个token
        否则返回None
        """
        curtk = self.curtoken()
        if curtk.kind != expected:
            return None
        for key, val in kwargs.items():
            if not hasattr(curtk, key) or getattr(curtk, key) != val:
                return None
        self.nexttoken()
        return curtk

    @memorize
    def identifier(self):
        return self.expect(TokenKind.IDENTIFIER)

    @memorize
    def string_literal(self):
        return self.expect(TokenKind.STRINGLITERAL)

    @memorize
    def newline(self):
        return self.expect(TokenKind.NEWLINE)

    @memorize
    def integer_constant(self):
        return self.expect(TokenKind.INTCONST)

    @memorize
    def floating_constant(self):
        return self.expect(TokenKind.FLOATCONST)

    @memorize
    def character_constant(self):
        return self.expect(TokenKind.CHARCONST)

    @memorize
    def end(self):
        return self.expect(TokenKind.END)
