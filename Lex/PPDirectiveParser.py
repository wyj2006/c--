from typing import TYPE_CHECKING
from Lex.PPFlag import PPFlag
from Parse import Parser
from AST import (
    Group,
    ElifDirecvtive,
)
from Basic import Diagnostic, Token, TokenKind, Error
from Parse.Builder import memorize

if TYPE_CHECKING:
    from Lex.Preprocessor import Preprocessor


from Lex.gen_PPDirectiveParser import Gen_PPDirectiveParser
from Parse import Parser


class PPDirectiveParser(Gen_PPDirectiveParser, Parser):
    tokengen: "Preprocessor"

    @memorize
    def pp_tokens(self) -> list[Token]:
        """
        pp-tokens[opt]:
            preprocessing-token
            pp-tokens preprocessing-token
        """
        tokens = []
        token = self.curtoken()
        while token.kind not in (TokenKind.NEWLINE, TokenKind.END):
            tokens.append(token)
            self.expect(token.kind)
            token = self.curtoken()
        return tokens

    @memorize
    def group(self):
        """
        group[opt]:
            group-part
            group group-part
        """
        group = Group(location=self.curtoken().location, parts=[])
        while (
            not (
                self.lookahead(TokenKind.HASH, TokenKind.ELIF)
                or self.lookahead(TokenKind.HASH, TokenKind.ELIFDEF)
                or self.lookahead(TokenKind.HASH, TokenKind.ELIFNDEF)
                or self.lookahead(TokenKind.HASH, TokenKind.ELSE)
                or self.lookahead(TokenKind.HASH, TokenKind.ENDIF)
            )
            and self.curtoken().kind != TokenKind.END
        ):
            part = self.group_part()
            if part == None:
                break
            if isinstance(part, list):
                group.parts.extend(part)
            else:
                group.parts.append(part)
        return group

    @memorize
    def pp_balanced_token(self) -> list[Token]:
        """
        pp-balanced-token:
            ( pp-balanced-token-sequence[opt] )
            [ pp-balanced-token-sequence[opt] ]
            { pp-balanced-token-sequence[opt] }
            any pp-token other than a parenthesis, a bracket, or a brace
        """
        z = self.save()
        if (
            (b := self.expect(TokenKind.L_PAREN))
            and ((a := self.pp_balanced_token_sequence()),)
            and (c := self.expect(TokenKind.R_PAREN))
        ):
            return [b] + a + [c]
        self.restore(z)
        if (
            (b := self.expect(TokenKind.L_SQUARE))
            and ((a := self.pp_balanced_token_sequence()),)
            and (c := self.expect(TokenKind.L_SQUARE))
        ):
            return [b] + a + [c]
        self.restore(z)
        if (
            (b := self.expect(TokenKind.L_BRACE))
            and ((a := self.pp_balanced_token_sequence()),)
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

    @memorize
    def header_name(self):
        return self.expect(TokenKind.HEADERNAME)

    @memorize
    def lparen(self):
        token = self.curtoken()
        if token.kind == TokenKind.L_PAREN and token.islparen:
            self.nexttoken()
            return token
        return None

    def defined_macro_expression(self):
        with self.tokengen.setFlag(remove_flags=PPFlag.ALLOW_REPLACE):
            return super().defined_macro_expression()

    def control_line(self):
        if self.lookahead(TokenKind.HASH, TokenKind.INCLUDE) or self.lookahead(
            TokenKind.HASH, TokenKind.EMBED
        ):
            with self.tokengen.setFlag(PPFlag.ALLOW_HEADERNAME | PPFlag.ALLOW_REPLACE):
                return super().control_line()
        return super().control_line()

    def has_include_expression(self):
        if self.lookahead(TokenKind.HAS_INCLUDE):
            with self.tokengen.setFlag(PPFlag.ALLOW_HEADERNAME):
                return super().has_include_expression()
        return super().has_include_expression()

    def has_embed_expression(self):
        if self.lookahead(TokenKind.HAS_EMBED):
            with self.tokengen.setFlag(PPFlag.ALLOW_HEADERNAME):
                return super().has_embed_expression()
        return super().has_embed_expression()

    def if_group(self):
        if self.lookahead(TokenKind.HASH, TokenKind.IF):
            with self.tokengen.setFlag(PPFlag.ALLOW_REPLACE):
                return super().if_group()
        return super().if_group()

    def elif_group(self):
        if self.lookahead(TokenKind.HASH, TokenKind.ELIF):
            self.expect(TokenKind.HASH, ispphash=True)
            token = self.curtoken()
            self.expect(TokenKind.ELIF)
            pp_directive = ElifDirecvtive(
                location=token.location,
                symtab=self.tokengen.symtab,
            )
            with self.tokengen.setFlag(PPFlag.ALLOW_REPLACE):
                try:
                    pp_directive.expr = self.constant_expression()
                except Diagnostic as e:
                    # 只有该指令被跳过时才会报错
                    pp_directive.diagnostic = e
                    self.pp_tokens()
            if self.curtoken().kind != TokenKind.NEWLINE:
                raise Error("缺少换行符", self.curtoken().location)
            self.expect(TokenKind.NEWLINE)
            pp_directive.group = self.group()
            return pp_directive
        return super().elif_group()
