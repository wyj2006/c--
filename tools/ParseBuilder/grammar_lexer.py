from basic import Token, TokenKind, Error
from lex import PPFlag, Preprocessor


class GrammarLexer(Preprocessor):
    def __init__(self, reader):
        super().__init__(reader)
        self.flag.remove(PPFlag.ALLOW_CONTACT)

    def get_new_token(self):
        ch, location = self.reader.next()
        if ch == "{":
            brace_count = 1
            text = ""
            ch, loc = self.reader.next()
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
                ch, loc = self.reader.next()
            return Token(TokenKind.ACTION, location, "{" + text + "}")
        elif ch == '"':
            ch, loc = self.reader.next()
            location.extend(loc)
            if ch == '"':
                ch, loc = self.reader.next()
                location.extend(loc)
                if ch == '"':
                    text = ""
                    ch, loc = self.reader.next()
                    while ch and text[-3:] != '"""':
                        text += ch
                        location.extend(loc)
                        ch, loc = self.reader.next()
                    if text[-3:] != '"""':
                        raise Error('\'"""\'未闭合', loc)
                    self.reader.back()
                    return Token(TokenKind.HEADER, location, '"""' + text)
                else:
                    self.reader.back()
                    self.reader.back()
            else:
                self.reader.back()
            self.reader.back()
        else:
            self.reader.back()
        return super().get_new_token()
