import unicodedata
from Basic import FileReader, TokenGen, Token, TokenKind, Error


class LexerBase(TokenGen):
    def __init__(self, reader: FileReader):
        super().__init__()
        self.reader = reader
        self.tokens: list[Token] = []  # 当前已经读到的token
        self.nexttk_index = 0  # 下一个token索引

    def curtoken(self) -> Token:
        return self.tokens[self.nexttk_index - 1]

    def back(self):
        assert self.nexttk_index > 0
        self.nexttk_index -= 1

    def save(self):
        return self.nexttk_index

    def restore(self, index):
        self.nexttk_index = index

    def next(self):
        # 除了负责提供token, 还要对它们进行保存
        if self.nexttk_index >= len(self.tokens):
            token = self.getNewToken()
            while token == None:
                self.reader.next()  # 忽略一个字符
                token = self.getNewToken()
            self.tokens.append(token)
        elif self.tokens[self.nexttk_index].kind == TokenKind.SUB_TOKENGEN:
            tokengen: TokenGen = self.tokens[self.nexttk_index].tokengen
            token = tokengen.next()
            if token.kind == TokenKind.END:
                self.tokens.pop(self.nexttk_index)
                return self.next()  # 防止下一个还是TokenGen
            else:
                self.tokens.insert(self.nexttk_index, token)
        token = self.tokens[self.nexttk_index]
        self.nexttk_index += 1
        return token

    def getNewToken(self): ...

    def error(self, msg, location):
        raise Error(msg, location)

    def other_identifier_start(self, ch: str):
        """
        identifier-start:
            nondigit
            XID_Start character
            universal character name of class XID_Start

        nondigit: one of
            _ a b c d e f g h i j k l m
            n o p q r s t u v w x y z
            A B C D E F G H I J K L M
            N O P Q R S T U V W X Y Z
        """
        try:
            if ch == "_" or ch.isalpha() or "XID_Start" in unicodedata.name(ch):
                return True
        except:
            pass
        return False

    identifier_start = other_identifier_start

    def other_identifier_continue(self, ch: str):
        """
        identifier-continue:
            digit
            nondigit
            XID_Continue character
            universal character name of class XID_Continue
        """
        try:
            if (
                ch == "_"
                or ch.isdigit()
                or ch.isalpha()
                or "XID_Continue" in unicodedata.name(ch)
            ):
                return True
        except:
            pass
        return False

    identifier_continue = other_identifier_continue

    def other_c_char(self, ch):
        """
        c-char:
            any member of the source character set except
                the single-quote ', backslash \, or new-line character
            escape-sequence
        """
        return ch not in "'\\\n"

    def other_s_char(self, ch):
        """
        s-char:
            any member of the source character set except
                the single-quote ", backslash \, or new-line character
            escape-sequence
        """
        return ch not in '"\\\n'
