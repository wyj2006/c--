from basic import FileReader, Location, Token, TokenGen, TokenKind


class EmbedLexer(TokenGen):
    def __init__(
        self,
        reader: FileReader,
        limit=float("inf"),
        prefix: list[Token] = None,
        suffix: list[Token] = None,
        if_empty: list[Token] = None,
    ):
        super().__init__()
        self.reader = reader
        self.limit = limit
        self.prefix = prefix if prefix != None else []
        self.suffix = suffix if suffix != None else []
        self.if_empty = if_empty if if_empty != None else []
        self.tokens: list[Token] = []
        self.nexttk_index = 0
        self.new_token_gen = None

    def next(self):
        if self.nexttk_index >= len(self.tokens):
            token = self.getNewToken()
            self.tokens.append(token)
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

    def getNewToken(self) -> Token:
        if self.new_token_gen == None:
            self.new_token_gen = self._getNewToken()
        try:
            return next(self.new_token_gen)
        except StopIteration:
            return Token(
                TokenKind.END,
                (
                    self.tokens[-1].location
                    if self.tokens
                    else Location(
                        [
                            {
                                "filename": self.reader.filename,
                                "lineno": 1,
                                "col": 1,
                                "span_col": 1,
                            }
                        ]
                    )
                ),
                "",
            )

    def _getNewToken(self):
        # 该生成器只会被创建一次
        if len(self.reader.lines) == 0:
            for i in self.if_empty:
                yield i
            return
        for i in self.prefix:
            yield i

        i = 0
        while i < self.limit:
            ch, location = self.reader.next()
            if ch == "":
                break
            if i >= 1:
                yield Token(TokenKind.COMMA, location, ",")
            yield Token(TokenKind.INTCONST, location, hex(ch))
            i += 1

        for i in self.suffix:
            yield i
