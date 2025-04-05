import unicodedata
from Basic import Token, TokenGen, TokenKind, FileReader, Location


class CharLexer(TokenGen):
    """将FileReader读取到的字符转换成Token"""

    def __init__(self, reader: FileReader):
        super().__init__()
        self.reader = reader

    def check_pphash(self):
        """判断当前读到的'#'是否算是预处理器指令"""
        i = self.reader.nextindex - 2
        atbegineofline = True  # 位于行首
        while i >= 0:
            if self.reader.hasread[i][0] == "\n":
                break
            if not self.reader.hasread[i][0].isspace():
                atbegineofline = False
                break
            i -= 1
        return atbegineofline

    def make_token(self, ch: str, location: Location):
        token_kind = TokenKind.IDENTIFIER
        if ch in Token.punctuator:
            token_kind = Token.punctuator[ch]
        elif ch == "":
            token_kind = TokenKind.END
        token = Token(token_kind, location, ch)
        if token.kind == TokenKind.HASH:
            token.ispphash = self.check_pphash()
        elif token.kind == TokenKind.L_PAREN:
            i = self.reader.nextindex - 2
            token.islparen = i >= 0 and (
                self.isIdentifierContinue(self.reader.hasread[i][0])
                or self.isIdentifierStart(self.reader.hasread[i][0])
            )
        return token

    def curtoken(self) -> Token:
        return self.make_token(*self.reader.current())

    def next(self) -> Token:
        return self.make_token(*self.reader.next())

    def save(self):
        return self.reader.nextindex

    def restore(self, index):
        self.reader.nextindex = index

    def isIdentifierStart(self, ch: str):
        """判断字符是否可以作为标识符开头"""
        try:
            return ch == "_" or ch.isalpha() or "XID_Start" in unicodedata.name(ch)
        except:
            return False

    def isIdentifierContinue(self, ch):
        """判断字符是否可以作为标识符后继"""
        try:
            return (
                ch == "_"
                or ch.isdigit()
                or ch.isalpha()
                or "XID_Continue" in unicodedata.name(ch)
            )
        except:
            return False
