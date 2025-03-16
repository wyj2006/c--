from typing import TYPE_CHECKING
from Lex.PPFlag import PPFlag
from Parse import Parser
from AST import (
    IfDirecvtive,
    ElseDirecvtive,
    LineDirecvtive,
    Embed,
    EmptyDirective,
    EndifDirecvtive,
    ErrorDirecvtive,
    IfdefDirecvtive,
    UndefDirective,
    DefineDirective,
    IfndefDirecvtive,
    Pragma,
    Include,
    WarningDirecvtive,
    IfSection,
    Group,
    PPParameter,
    PrefixParam,
    LimitParam,
    SuffixParam,
    IfEmptyParam,
    Defined,
    HasCAttribute,
    HasEmbed,
    HasInclude,
    ElifDirecvtive,
)
from Basic import Diagnostic, Token, TokenKind, Error, Warn

if TYPE_CHECKING:
    from Lex.Preprocessor import Preprocessor


class PPDirectiveParser(Parser):
    tokengen: "Preprocessor"

    def start(self):
        pp_directive = self.group_part()
        if isinstance(pp_directive, list):
            # pp_directive[0]是'#'
            raise Error("未知的预处理指令", pp_directive[1].location)
        return pp_directive

    def expect(self, expected: TokenKind, **kwargs):
        curtk = self.curtoken()
        token = super().expect(expected, **kwargs)
        if token == None:
            # TODO
            return token
            raise Error(f"期待得到{expected.value}", curtk.location)
        return token

    def primary_expression(self):
        """
        defined-macro-expression:
            defined identifier
            defined ( identifier )
        header-name-tokens:
            string-literal
            < h-pp-tokens >
        has-include-expression:
            __has_include ( header-name )
            __has_include ( header-name-tokens )
        has-embed-expression:
            __has_embed ( header-name embed-parameter-sequence[opt] )
            __has_embed ( header-name-tokens pp-balanced-token-sequence[opt] )
        has-c-attribute-express:
            __has_c_attribute ( pp-tokens )
        """
        token = self.curtoken()
        match token.kind:
            case TokenKind.DEFINED:
                with self.tokengen.setFlag(remove_flags=PPFlag.ALLOW_REPLACE):
                    a = Defined(location=token.location)
                    self.expect(TokenKind.DEFINED)
                    if self.curtoken().kind == TokenKind.IDENTIFIER:
                        a.name = self.expect(TokenKind.IDENTIFIER).text
                    elif self.curtoken().kind == TokenKind.L_PAREN:
                        self.expect(TokenKind.L_PAREN)
                        a.name = self.expect(TokenKind.IDENTIFIER).text
                        self.expect(TokenKind.R_PAREN)
                    else:
                        raise Error("期待得到标识符", token.location)
                return a
            case TokenKind.HAS_INCLUDE | TokenKind.HAS_EMBED:
                if token.kind == TokenKind.HAS_INCLUDE:
                    a = HasInclude(location=token.location)
                elif token.kind == TokenKind.HAS_EMBED:
                    a = HasEmbed(location=token.location)
                with self.tokengen.setFlag(PPFlag.ALLOW_HEADERNAME):
                    self.expect(token.kind)
                    self.expect(TokenKind.L_PAREN)
                    token = self.curtoken()
                    if token.kind == TokenKind.STRINGLITERAL:
                        a.filename = token.content
                        a.search_current_path = True
                    elif token.kind == TokenKind.HEADERNAME:
                        a.filename = token.text
                        a.search_current_path = False
                    else:
                        raise Error('期待 "文件名" 或 <文件名>', token.location)
                    self.expect(token.kind)
                if self.curtoken().kind == TokenKind.IDENTIFIER and isinstance(
                    a, HasEmbed
                ):
                    a.parameters = self.embed_parameter_sequence()
                self.expect(TokenKind.R_PAREN)
                return a
            case TokenKind.HAS_C_ATTRIBUTE:
                a = HasCAttribute(location=token.location)
                self.expect(TokenKind.HAS_C_ATTRIBUTE)
                self.expect(TokenKind.L_PAREN)
                a.attribute = self.attribute()
                self.expect(TokenKind.R_PAREN)
                return a
        return super().primary_expression()

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

    def control_line(self):
        """
        control-line:
            # include pp-tokens new-line
            # embed pp-tokens new-line
            # define identifier replacement-list new-line
            # define identifier lparen identifier-list[opt] ) replacement-list new-line
            # define identifier lparen ... ) replacement-list new-line
            # define identifier lparen identifier-list , ... ) replacement-list new-line
            # undef identifier new-line
            # line pp-tokens new-line
            # error pp-tokens[opt] new-line
            # warning pp-tokens[opt] new-line
            # pragma pp-tokens[opt] new-line
            # new-line

        identifier-list:
            identifier
            identifier-list , identifier

        replacement-list:
             pp-tokens[opt]
        """
        self.expect(TokenKind.HASH, ispphash=True)
        token = self.curtoken()
        match (token.kind):
            case TokenKind.INCLUDE | TokenKind.EMBED:
                if token.kind == TokenKind.EMBED:
                    pp_directive = Embed(location=token.location)
                elif token.kind == TokenKind.INCLUDE:
                    pp_directive = Include(location=token.location)

                with self.tokengen.setFlag(
                    PPFlag.ALLOW_HEADERNAME | PPFlag.ALLOW_REPLACE
                ):
                    # expect会自动向后读, 所以要提前设置flag
                    self.expect(token.kind)
                    z = self.save()
                    pp_tokens = self.pp_tokens()
                y = self.save()
                self.restore(z)

                token = self.curtoken()
                if token.kind == TokenKind.STRINGLITERAL:
                    self.expect(TokenKind.STRINGLITERAL)
                    pp_directive.filename = token.content
                    pp_directive.search_current_path = True
                elif token.kind == TokenKind.HEADERNAME:
                    self.expect(TokenKind.HEADERNAME)
                    pp_directive.filename = token.text
                    pp_directive.search_current_path = False
                else:
                    raise Error('期待 "文件名" 或 <文件名>', token.location)

                if self.curtoken().kind == TokenKind.IDENTIFIER and isinstance(
                    pp_directive, Embed
                ):
                    pp_directive.parameters = self.embed_parameter_sequence()

                if self.save() != y:
                    Warn(
                        f"在{token.kind.value}指令后面的多余的token",
                        pp_tokens[1].location,
                    ).dump()

                self.restore(y)
                self.expect(TokenKind.NEWLINE)
                return pp_directive
            case TokenKind.DEFINE:
                pp_directive = DefineDirective(
                    location=token.location,
                    parameters=[],
                    hasvarparam=False,
                    is_object_like=True,
                    replacement=[],
                )

                self.expect(TokenKind.DEFINE)
                name_token = self.expect(TokenKind.IDENTIFIER)
                pp_directive.name = name_token.text

                token = self.curtoken()
                if token.kind == TokenKind.L_PAREN and token.islparen:
                    pp_directive.is_object_like = False

                    self.expect(TokenKind.L_PAREN, islparen=True)

                    end_tokens = (
                        TokenKind.R_PAREN,
                        TokenKind.END,
                        TokenKind.NEWLINE,
                    )
                    token = self.curtoken()
                    if token.kind == TokenKind.IDENTIFIER:
                        self.expect(TokenKind.IDENTIFIER)
                        pp_directive.parameters.append(token.text)
                    elif token.kind == TokenKind.ELLIPSIS:
                        self.expect(TokenKind.ELLIPSIS)
                        pp_directive.hasvarparam = True
                    elif token.kind not in end_tokens:
                        raise Error(
                            f"宏参数列表中不合法的token: {token.text}",
                            token.location,
                        )

                    token = self.curtoken()
                    while (
                        token.kind == TokenKind.COMMA and not pp_directive.hasvarparam
                    ):
                        self.expect(TokenKind.COMMA)

                        token = self.curtoken()
                        if token.kind == TokenKind.IDENTIFIER:
                            self.expect(TokenKind.IDENTIFIER)
                            pp_directive.parameters.append(token.text)
                        elif token.kind == TokenKind.ELLIPSIS:
                            self.expect(TokenKind.ELLIPSIS)
                            pp_directive.hasvarparam = True
                            break
                        else:
                            raise Error(
                                f"宏参数列表中不合法的token: {token.text}",
                                token.location,
                            )
                        token = self.curtoken()

                    self.expect(TokenKind.R_PAREN)

                pp_directive.replacement = self.pp_tokens()

                self.expect(TokenKind.NEWLINE)
                return pp_directive
            case TokenKind.UNDEF:
                self.expect(TokenKind.UNDEF)
                name_token = self.expect(TokenKind.IDENTIFIER)
                self.expect(TokenKind.NEWLINE)
                return UndefDirective(location=token.location, name=name_token.text)
            case TokenKind.LINE:
                pp_directive = LineDirecvtive(location=token.location)

                self.expect(TokenKind.LINE)
                pp_tokens = self.pp_tokens()
                if (
                    not pp_tokens
                    or pp_tokens[0].kind != TokenKind.INTCONST
                    or pp_tokens[0].text[0] == "-"
                ):
                    raise Error("期望一个非负整数", token.location)
                elif pp_tokens:
                    pp_directive.lineno = int(pp_tokens[0].text)

                if len(pp_tokens) == 2:
                    if pp_tokens[1].kind == TokenKind.STRINGLITERAL:
                        pp_directive.filename = pp_tokens[1].content
                    else:
                        raise Error(
                            f"不合法的文件名: {pp_tokens[1].text}",
                            pp_tokens[1].location,
                        )

                if len(pp_tokens) > 2:
                    Warn(
                        f"多余的token: {pp_tokens[2].text}", pp_tokens[2].location
                    ).dump()

                self.expect(TokenKind.NEWLINE)
                return pp_directive
            case TokenKind.ERROR | TokenKind.WARNING:
                if token.kind == TokenKind.ERROR:
                    pp_directive = ErrorDirecvtive(location=token.location)
                elif token.kind == TokenKind.WARNING:
                    pp_directive = WarningDirecvtive(location=token.location)
                self.expect(token.kind)
                pp_directive.messages = self.pp_tokens()
                self.expect(TokenKind.NEWLINE)
                return pp_directive
            case TokenKind.PRAGMA:
                pp_directive = Pragma(location=token.location)
                self.expect(TokenKind.PRAGMA)
                pp_directive.args = self.pp_tokens()
                self.expect(TokenKind.NEWLINE)
                return pp_directive
            case TokenKind.NEWLINE:
                self.expect(TokenKind.NEWLINE)
                return EmptyDirective(location=token.location)
            case _:
                raise Error(f"未知的预处理指令: {token.text}", token.location)

    def if_section(self):
        """
        if-section:
            if-group elif-groups[opt] else-group[opt] endif-line
        """
        pp_directive: IfSection = self.if_group()

        else_groups = self.elif_groups()
        if self.lookahead(TokenKind.HASH, TokenKind.ELSE):
            else_groups.append(self.else_group())
        p = pp_directive
        for else_group in else_groups:
            p.else_group = else_group
            p = p.else_group

        self.endif_line()
        return pp_directive

    def if_group(self):
        """
        if-group:
            # if constant-expression new-line group[opt]
            # ifdef identifier new-line group[opt]
            # ifndef identifier new-line group[opt]
        """
        self.expect(TokenKind.HASH, ispphash=True)
        token = self.curtoken()
        match (token.kind):
            case TokenKind.IF:
                pp_directive = IfDirecvtive(
                    location=token.location, macros=self.tokengen.macros
                )
                with self.tokengen.setFlag(PPFlag.ALLOW_REPLACE):
                    self.expect(TokenKind.IF)
                    pp_directive.expr = self.constant_expression()
                self.expect(TokenKind.NEWLINE)
                pp_directive.group = self.group()
                return pp_directive
            case TokenKind.IFDEF | TokenKind.IFNDEF:
                if token.kind == TokenKind.IFDEF:
                    pp_directive = IfdefDirecvtive(
                        location=token.location, macros=self.tokengen.macros
                    )
                elif token.kind == TokenKind.IFNDEF:
                    pp_directive = IfndefDirecvtive(
                        location=token.location, macros=self.tokengen.macros
                    )
                self.expect(token.kind)
                token = self.expect(TokenKind.IDENTIFIER)
                pp_directive.name = token.text
                self.expect(TokenKind.NEWLINE)
                pp_directive.group = self.group()
                return pp_directive
            case _:
                raise Error(f"未知的条件预处理指令: {token.text}", token.location)

    def elif_groups(self):
        """
        elif-groups:
            elif-group
            elif-groups elif-group
        """
        group = []
        while (
            self.lookahead(TokenKind.HASH, TokenKind.ELIF)
            or self.lookahead(TokenKind.HASH, TokenKind.ELIFDEF)
            or self.lookahead(TokenKind.HASH, TokenKind.ELIFNDEF)
        ):
            group.append(self.elif_group())
        return group

    def elif_group(self):
        """
        elif-group[opt]:
            # elif constant-expression new-line group[opt]
            # elifdef identifier new-line group[opt]
            # elifndef identifier new-line group[opt]
        """
        self.expect(TokenKind.HASH, ispphash=True)
        token = self.curtoken()
        match (token.kind):
            case TokenKind.ELIF:
                pp_directive = ElifDirecvtive(
                    location=token.location, macros=self.tokengen.macros
                )
                with self.tokengen.setFlag(PPFlag.ALLOW_REPLACE):
                    try:
                        self.expect(TokenKind.ELIF)
                        pp_directive.expr = self.constant_expression()
                    except Diagnostic as e:
                        # 只有该指令被跳过时才会报错
                        pp_directive.diagnostic = e
                        self.pp_tokens()
                self.expect(TokenKind.NEWLINE)
                pp_directive.group = self.group()
                return pp_directive
            case TokenKind.ELIFDEF | TokenKind.ELIFNDEF:
                if token.kind == TokenKind.ELIFDEF:
                    pp_directive = IfdefDirecvtive(
                        location=token.location, macros=self.tokengen.macros
                    )
                elif token.kind == TokenKind.ELIFNDEF:
                    pp_directive = IfndefDirecvtive(
                        location=token.location, macros=self.tokengen.macros
                    )
                self.expect(token.kind)
                token = self.expect(TokenKind.IDENTIFIER)
                pp_directive.name = token.text
                self.expect(TokenKind.NEWLINE)
                pp_directive.group = self.group()
                return pp_directive
            case _:
                raise Error(f"未知的条件预处理指令: {token.text}", token.location)

    def else_group(self):
        """
        else_group:
            # else new-line group[opt]
        """
        self.expect(TokenKind.HASH, ispphash=True)
        token = self.expect(TokenKind.ELSE)
        pp_directive = ElseDirecvtive(location=token.location)
        self.expect(TokenKind.NEWLINE)
        pp_directive.group = self.group()
        return pp_directive

    def endif_line(self):
        """
        endif-line:
            # endif new-line
        """
        self.expect(TokenKind.HASH, ispphash=True)
        token = self.expect(TokenKind.ENDIF)
        self.expect(TokenKind.NEWLINE)
        return EndifDirecvtive(location=token.location)

    def group(self):
        """
        group[opt]:
            group-part
            group group-part
        """
        group = Group(location=self.curtoken().location, parts=[])
        while not (
            self.lookahead(TokenKind.HASH, TokenKind.ELIF)
            or self.lookahead(TokenKind.HASH, TokenKind.ELIFDEF)
            or self.lookahead(TokenKind.HASH, TokenKind.ELIFNDEF)
            or self.lookahead(TokenKind.HASH, TokenKind.ELSE)
            or self.lookahead(TokenKind.HASH, TokenKind.ENDIF)
        ):
            part = self.group_part()
            if isinstance(part, list):
                group.parts.extend(part)
            else:
                group.parts.append(part)
        return group

    def group_part(self):
        """
        group_part:
            if-section
            control-line
            text-line
            # non-directive

        text-line:
            pp-tokens[opt] new-line

        non-directive:
            pp-tokens new-line
        """
        if (
            self.lookahead(TokenKind.HASH, TokenKind.IF)
            or self.lookahead(TokenKind.HASH, TokenKind.IFDEF)
            or self.lookahead(TokenKind.HASH, TokenKind.IFNDEF)
        ):
            return self.if_section()
        elif (
            self.lookahead(TokenKind.HASH, TokenKind.INCLUDE)
            or self.lookahead(TokenKind.HASH, TokenKind.EMBED)
            or self.lookahead(TokenKind.HASH, TokenKind.DEFINE)
            or self.lookahead(TokenKind.HASH, TokenKind.UNDEF)
            or self.lookahead(TokenKind.HASH, TokenKind.LINE)
            or self.lookahead(TokenKind.HASH, TokenKind.ERROR)
            or self.lookahead(TokenKind.HASH, TokenKind.WARNING)
            or self.lookahead(TokenKind.HASH, TokenKind.PRAGMA)
            or self.lookahead(TokenKind.HASH, TokenKind.NEWLINE)
        ):
            return self.control_line()
        else:
            pp_tokens = self.pp_tokens()
            self.expect(TokenKind.NEWLINE)
            return pp_tokens

    def pp_parameter_name(self) -> tuple[str, str]:
        """
        pp-parameter-name:
            pp-standard-parameter
            pp-prefixed-parameter

        pp-standard-parameter:
            identifier

        pp-prefixed-parameter:
            identifier :: identifier
        """
        token = self.expect(TokenKind.IDENTIFIER)
        prefix = ""
        name = token.text
        if self.curtoken().kind == TokenKind.COLONCOLON:
            self.expect(TokenKind.COLONCOLON)
            prefix = name
            token = self.expect(TokenKind.IDENTIFIER)
            name = token.text
        return (prefix, name)

    def pp_parameter(self):
        """
        pp-parameter:
            pp-parameter-name pp-parameter-clause[opt]
        """
        token = self.curtoken()
        match token.text:
            case "limit" | "__limit__":
                pp_directive = LimitParam(location=token.location)
                self.expect(TokenKind.IDENTIFIER)
                self.expect(TokenKind.L_PAREN)
                pp_directive.expr = self.constant_expression()
                self.expect(TokenKind.R_PAREN)
            case _:
                standard_param = {
                    "suffix": SuffixParam,
                    "__suffix__": SuffixParam,
                    "prefix": PrefixParam,
                    "__prefix__": PrefixParam,
                    "if_empty": IfEmptyParam,
                    "__if_empty__": IfEmptyParam,
                }
                pp_directive = PPParameter(location=token.location, args=[])
                pp_directive.prefix, pp_directive.name = self.pp_parameter_name()
                if self.curtoken().kind == TokenKind.L_PAREN:
                    pp_directive.args = self.pp_parameter_clause()
                if pp_directive.prefix == "" and pp_directive.name in standard_param:
                    pp_directive = standard_param[pp_directive.name](
                        **pp_directive.__dict__
                    )
        return pp_directive

    def pp_parameter_clause(self) -> list[Token]:
        """
        pp-parameter-clause:
            ( pp-balanced-token-sequence[opt] )
        """
        tokens = []
        self.expect(TokenKind.L_PAREN)
        if self.curtoken().kind != TokenKind.R_PAREN:
            tokens = self.pp_balanced_token_sequence()
        self.expect(TokenKind.R_PAREN)
        return tokens

    def pp_balanced_token_sequence(self) -> list[Token]:
        """
        pp-balanced-token-sequence:
            pp-balanced-token
            pp-balanced-token-sequence pp-balanced-token
        """
        tokens = []
        tokens.extend(self.pp_balanced_token())
        while self.curtoken().kind not in (
            TokenKind.R_BRACE,
            TokenKind.R_PAREN,
            TokenKind.R_SQUARE,
        ):
            tokens.extend(self.pp_balanced_token())
        return tokens

    def pp_balanced_token(self) -> list[Token]:
        """
        pp-balanced-token:
            ( pp-balanced-token-sequence[opt] )
            [ pp-balanced-token-sequence[opt] ]
            { pp-balanced-token-sequence[opt] }
            any pp-token other than a parenthesis, a bracket, or a brace
        """
        token = self.curtoken()
        match (token.kind):
            case TokenKind.L_PAREN:
                self.expect(TokenKind.L_PAREN)
                if self.curtoken().kind != TokenKind.R_PAREN:
                    tokens = self.pp_balanced_token_sequence()
                self.expect(TokenKind.R_PAREN)
            case TokenKind.L_BRACE:
                self.expect(TokenKind.L_BRACE)
                if self.curtoken().kind != TokenKind.R_BRACE:
                    tokens = self.pp_balanced_token_sequence()
                self.expect(TokenKind.R_BRACE)
            case TokenKind.L_SQUARE:
                self.expect(TokenKind.L_SQUARE)
                if self.curtoken().kind != TokenKind.R_SQUARE:
                    tokens = self.pp_balanced_token_sequence()
                self.expect(TokenKind.R_SQUARE)
            case _:
                self.expect(token.kind)
                tokens = [token]
        return tokens

    def embed_parameter_sequence(self):
        """
        embed-parameter-sequence:
            pp-parameter
            embed-parameter-sequence pp-parameter
        """
        parameters: list[PPParameter] = []
        parameters.append(self.pp_parameter())
        while self.curtoken().kind == TokenKind.IDENTIFIER:
            parameters.append(self.pp_parameter())
        return parameters
