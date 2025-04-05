from typing import Union
import unicodedata

from Basic import Token, TokenGen, TokenKind, FileReader
from Lex.CharLexer import CharLexer
from Parse.Builder import memorize

from Lex.gen_LexerParser import Gen_LexerParser


class LexerParser(Gen_LexerParser, TokenGen):
    def next(self):
        token = self.start()
        while token == None:
            self.nexttoken()
            token = self.start()
        return token

    @memorize
    def digit(self):
        """
        digit: one of
            0 1 2 3 4 5 6 7 8 9
        """
        token = self.curtoken()
        ch = token.text
        if ch and ch.isdigit():
            self.nexttoken()
            return token
        return None

    @memorize
    def identifier_start(self):
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
        token = self.curtoken()
        ch = token.text
        try:
            if ch == "_" or ch.isalpha() or "XID_Start" in unicodedata.name(ch):
                self.nexttoken()
                return token
        except:
            pass
        return None

    @memorize
    def identifier_continue(self):
        """
        identifier-continue:
            digit
            nondigit
            XID_Continue character
            universal character name of class XID_Continue
        """
        token = self.curtoken()
        ch = token.text
        try:
            if (
                ch == "_"
                or ch.isdigit()
                or ch.isalpha()
                or "XID_Continue" in unicodedata.name(ch)
            ):
                self.nexttoken()
                return token
        except:
            pass
        return None

    @memorize
    def nonzero_digit(self):
        """
        nonzero-digit: one of
            1 2 3 4 5 6 7 8 9
        """
        token = self.curtoken()
        ch = token.text
        if ch and ch in "123456789":
            self.nexttoken()
            return token
        return None

    @memorize
    def octal_digit(self):
        """
        octal-digit: one of
            0 1 2 3 4 5 6 7
        """
        token = self.curtoken()
        ch = token.text
        if ch and ch in "01234567":
            self.nexttoken()
            return token
        return None

    @memorize
    def hexadecimal_digit(self):
        """
        hexadecimal-digit: one of
            0 1 2 3 4 5 6 7 8 9
            a b c d e f
            A B C D E F
        """
        token = self.curtoken()
        ch = token.text
        if ch and ch in "0123456789abcdefABCDEF":
            self.nexttoken()
            return token
        return None

    @memorize
    def binary_digit(self):
        """
        binary-digit: one of
            0 1
        """
        token = self.curtoken()
        ch = token.text
        if ch and ch in "01":
            self.nexttoken()
            return token
        return None

    @memorize
    def unsigned_suffix(self):
        """
        unsigned-suffix: one of
            u U
        """
        token = self.curtoken()
        ch = token.text
        if ch and ch in "uU":
            self.nexttoken()
            return token
        return None

    @memorize
    def long_suffix(self):
        """
        long-suffix: one of
            l L
        """
        token = self.curtoken()
        ch = token.text
        if ch and ch in "lL":
            self.nexttoken()
            return token
        return None

    @memorize
    def c_char(self):
        """
        c-char:
            any member of the source character set except
                the single-quote ', backslash \, or new-line character
            escape-sequence
        """
        _z = self.save()
        if a := self.escape_sequence():
            return a
        self.restore(_z)

        token = self.curtoken()
        ch = token.text
        if ch not in "'\\\n":
            self.nexttoken()
            return token
        return None

    @memorize
    def s_char(self):
        """
        s-char:
            any member of the source character set except
                the single-quote ", backslash \, or new-line character
            escape-sequence
        """
        _z = self.save()
        if a := self.escape_sequence():
            return a
        self.restore(_z)

        token = self.curtoken()
        ch = token.text
        if ch not in '"\\\n':
            self.nexttoken()
            return token
        return None


class Lexer(TokenGen):
    def __init__(self, reader: FileReader):
        super().__init__()
        self.reader = reader
        self.charlexer = CharLexer(reader)
        self.lexerparser = LexerParser(self.charlexer)
        self.is_lexerparser_init = False

        self.tokens: list[Union[Token, TokenGen]] = []  # 当前已经读到的token
        self.nexttk_index = 0  # 下一个token索引

    def curtoken(self) -> Token:
        return self.tokens[self.nexttk_index - 1]

    def next(self):
        # 除了负责提供token, 还要对它们进行保存
        if self.nexttk_index >= len(self.tokens):
            if not self.is_lexerparser_init:
                self.is_lexerparser_init = True
                self.lexerparser.nexttoken()
            token = self.lexerparser.next()
            self.tokens.append(token)
        elif isinstance(self.tokens[self.nexttk_index], TokenGen):
            tokengen: TokenGen = self.tokens[self.nexttk_index]
            token = tokengen.next()
            if token.kind == TokenKind.END:
                self.tokens.pop(self.nexttk_index)
                return self.next()  # 防止下一个还是TokenGen
            else:
                self.tokens.insert(self.nexttk_index, token)
        token = self.tokens[self.nexttk_index]
        self.nexttk_index += 1
        return token

    def back(self):
        assert self.nexttk_index > 0
        self.nexttk_index -= 1

    def save(self):
        return self.nexttk_index

    def restore(self, index):
        self.nexttk_index = index
