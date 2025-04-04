from Basic import Token, TokenKind, Error
from Lex import PPFlag, Preprocessor


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
