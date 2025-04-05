from Basic import Location, Token, TokenKind, Error
from Lex import PPFlag, Preprocessor
from .Memory import memorize


class GrammarLexer(Preprocessor):
    def __init__(self, reader):
        super().__init__(reader)
        self.flag.remove(PPFlag.ALLOW_CONTACT)

    def getNewToken(self):
        ch, location = self.getch()
        if ch == "{":
            brace_count = 1
            text = ""
            ch, loc = self.getch()
            while True:
                if ch == "}":
                    brace_count -= 1
                    if brace_count == 0:
                        break
                elif ch == "":
                    raise Error("'{'未闭合", loc)
                elif ch == "{":
                    brace_count += 1
                text += ch
                location.extend(loc)
                ch, loc = self.getch()
            return Token(TokenKind.ACTION, location, text)
        elif ch == '"':
            back_count = 1
            ch, loc = self.getch()
            location.extend(loc)
            if ch == '"':
                ch, loc = self.getch()
                location.extend(loc)
                if ch == '"':
                    text = ""
                    ch, loc = self.getch()
                    while ch and text[-3:] != '"""':
                        text += ch
                        location.extend(loc)
                        ch, loc = self.getch()
                    if text[-3:] != '"""':
                        raise Error('\'"""\'未闭合', loc)
                    self.ungetch()
                    return Token(TokenKind.HEADER, location, text[:-3])
                else:
                    back_count += 1
            else:
                back_count += 1
            for i in range(back_count):
                self.ungetch()
        else:
            self.ungetch()
        return super().getNewToken()


from .gen_GrammarLexerParser import Gen_GrammarLexerParser
from Lex.Preprocessor import PPLexerParser


class GramarLexerParser(Gen_GrammarLexerParser, PPLexerParser):
    @memorize
    def header_chars(self):
        header_token = Token(TokenKind.HEADER, Location([]), "")
        while self.curtoken().kind != TokenKind.END:
            _z = self.save()

            token = self.curtoken()
            ch = token.text
            if ch == '"':
                self.nexttoken()
                token = self.curtoken()
                ch = token.text
                if ch == '"':
                    self.nexttoken()
                    token = self.curtoken()
                    ch = token.text
                    if ch == '"':
                        self.restore(_z)
                        break
            self.restore(_z)

            token = self.curtoken()
            header_token.location.extend(token.location)
            header_token.text += token.text
            self.nexttoken()
        return header_token

    @memorize
    def action_chars(self):
        action_token = Token(TokenKind.ACTION, Location([]), "")
        while self.curtoken().kind != TokenKind.END:
            _z = self.save()

            token = self.curtoken()
            ch = token.text
            if ch == "}":
                self.restore(_z)
                break
            self.restore(_z)

            token = self.curtoken()
            action_token.location.extend(token.location)
            action_token.text += token.text
            self.nexttoken()
        return action_token


class GrammarLexer(Preprocessor):
    def __init__(self, reader):
        super().__init__(reader)
        self.lexerparser = GramarLexerParser(self.charlexer)
        self.flag.remove(PPFlag.ALLOW_CONTACT)
