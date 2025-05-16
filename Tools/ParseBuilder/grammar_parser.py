from basic import TokenKind
from parse.memory import memorize
from .gen_builder_parser import Gen_BuilderParser


class GrammarParser(Gen_BuilderParser):
    @memorize
    def action(self):
        return self.expect(TokenKind.ACTION)

    @memorize
    def header(self):
        return self.expect(TokenKind.HEADER)

    def item(self):
        if self.lookahead(TokenKind.IDENTIFIER, TokenKind.COLON):
            return None
        return super().item()
