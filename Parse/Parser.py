from Basic import Token, TokenGen, TokenKind
from Parse.Builder.Memory import memorize
from Parse.Wrapper import may_update_type_symbol, may_enter_scope

from .gen_CParser import Gen_CParser


class Parser(Gen_CParser):
    def __init__(self, tokengen: TokenGen):
        super().__init__(tokengen)
        self.type_symbol: list[str] = []  # 种类为类型的符号

    @memorize
    def identifier(self):
        a = self.expect(TokenKind.IDENTIFIER)
        if a != None and a.text not in self.type_symbol:
            return a
        return None

    @memorize
    def typedef_name(self):
        a = self.expect(TokenKind.IDENTIFIER)
        if a != None and a.text in self.type_symbol:
            return a
        return None

    @may_update_type_symbol
    def declaration(self):
        return super().declaration()

    @may_enter_scope
    def compound_statement(self):
        return super().compound_statement()

    @memorize
    def balanced_token(self) -> list[Token]:
        """
        balanced-token:
            ( balanced-token-sequence[opt] )
            [ balanced-token-sequence[opt] ]
            { balanced-token-sequence[opt] }
            any token other than a parenthesis, a bracket, or a brace
        """
        z = self.save()
        if (
            (b := self.expect(TokenKind.L_PAREN))
            and ((a := self.balanced_token_sequence()),)
            and (c := self.expect(TokenKind.R_PAREN))
        ):
            return [b] + a + [c]
        self.restore(z)
        if (
            (b := self.expect(TokenKind.L_SQUARE))
            and ((a := self.balanced_token_sequence()),)
            and (c := self.expect(TokenKind.L_SQUARE))
        ):
            return [b] + a + [c]
        self.restore(z)
        if (
            (b := self.expect(TokenKind.L_BRACE))
            and ((a := self.balanced_token_sequence()),)
            and (c := self.expect(TokenKind.R_BRACE))
        ):
            return [b] + a + [c]
        self.restore(z)
        if self.curtoken().kind not in (
            TokenKind.R_PAREN,
            TokenKind.R_SQUARE,
            TokenKind.R_BRACE,
        ):
            return [self.expect(self.curtoken().kind)]
        self.restore(z)
        return None
