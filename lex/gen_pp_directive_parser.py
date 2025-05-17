from parse import ParserBase, memorize, memorize_left_rec
from basic import *
from cast import *

def reconstruct(if_group, else_groups):
    p = if_group
    for else_group in else_groups:
        p.else_group = else_group
        p = p.else_group
    return if_group

class Gen_PPDirectiveParser(ParserBase):

    @memorize
    def start(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (group_part := self.group_part()):
            return group_part
        self.restore(_z)
        return None

    @memorize
    def conditional_inclusion(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (defined_macro_expression := self.defined_macro_expression()):
            return defined_macro_expression
        self.restore(_z)
        if (has_include_expression := self.has_include_expression()):
            return has_include_expression
        self.restore(_z)
        if (has_embed_expression := self.has_embed_expression()):
            return has_embed_expression
        self.restore(_z)
        if (has_c_attribute_express := self.has_c_attribute_express()):
            return has_c_attribute_express
        self.restore(_z)
        return None

    @memorize
    def defined_macro_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.DEFINED)) and (b := self.identifier()):
            return Defined(location=a.location, name=b.text)
        self.restore(_z)
        if (a := self.expect(TokenKind.DEFINED)) and self.expect(TokenKind.L_PAREN) and (b := self.identifier()) and self.expect(TokenKind.R_PAREN):
            return Defined(location=a.location, name=b.text)
        self.restore(_z)
        if (a := self.expect(TokenKind.DEFINED)) and self.expect(TokenKind.L_PAREN) and (b := self.identifier()):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.DEFINED)) and self.expect(TokenKind.L_PAREN):
            return self.error('期待标识符', self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.DEFINED)):
            return self.error('期待标识符', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def has_include_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.HAS_INCLUDE)) and self.expect(TokenKind.L_PAREN) and (b := self.header_name()) and self.expect(TokenKind.R_PAREN):
            return HasInclude(location=a.location, filename=b.text, search_current_path=False)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_INCLUDE)) and self.expect(TokenKind.L_PAREN) and (b := self.string_literal()) and self.expect(TokenKind.R_PAREN):
            return HasInclude(location=a.location, filename=b.content, search_current_path=True)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_INCLUDE)) and self.expect(TokenKind.L_PAREN) and (b := self.header_name()):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_INCLUDE)) and self.expect(TokenKind.L_PAREN) and (b := self.string_literal()):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_INCLUDE)) and self.expect(TokenKind.L_PAREN):
            return self.error('期待"文件名"或<文件名>', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HAS_INCLUDE):
            return self.error("期待'('", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def has_embed_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.HAS_EMBED)) and self.expect(TokenKind.L_PAREN) and (b := self.header_name()) and ((c := self.embed_parameter_sequence()),) and self.expect(TokenKind.R_PAREN):
            return HasEmbed(location=a.location, filename=b.text, search_current_path=False, parameters=c)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_EMBED)) and self.expect(TokenKind.L_PAREN) and (b := self.string_literal()) and ((c := self.embed_parameter_sequence()),) and self.expect(TokenKind.R_PAREN):
            return HasEmbed(location=a.location, filename=b.content, search_current_path=True, parameters=c)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_EMBED)) and self.expect(TokenKind.L_PAREN) and (b := self.header_name()) and ((embed_parameter_sequence := self.embed_parameter_sequence()),):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_EMBED)) and self.expect(TokenKind.L_PAREN) and (b := self.string_literal()) and ((embed_parameter_sequence := self.embed_parameter_sequence()),):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_EMBED)) and self.expect(TokenKind.L_PAREN):
            return self.error('期待"文件名"或<文件名>', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HAS_EMBED):
            return self.error("期待'('", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def has_c_attribute_express(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.HAS_C_ATTRIBUTE)) and self.expect(TokenKind.L_PAREN) and (b := self.attribute()) and self.expect(TokenKind.R_PAREN):
            return HasCAttribute(location=a.location, attribute=b)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_C_ATTRIBUTE)) and self.expect(TokenKind.L_PAREN) and (b := self.attribute()):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.HAS_C_ATTRIBUTE)) and self.expect(TokenKind.L_PAREN):
            return self.error('期待属性', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HAS_C_ATTRIBUTE):
            return self.error("期待'('", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def control_line(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.INCLUDE)) and (b := self.header_name()) and ((pp_tokens := self.pp_tokens()),) and (new_line := self.new_line()):
            return Include(location=a.location, filename=b.text, search_current_path=False)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.INCLUDE)) and (b := self.string_literal()) and ((pp_tokens := self.pp_tokens()),) and (new_line := self.new_line()):
            return Include(location=a.location, filename=b.content, search_current_path=True)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.EMBED)) and (b := self.header_name()) and ((c := self.embed_parameter_sequence()),) and ((pp_tokens := self.pp_tokens()),) and (new_line := self.new_line()):
            return Embed(location=a.location, filename=b.text, search_current_path=False, parameters=c)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.EMBED)) and (b := self.string_literal()) and ((c := self.embed_parameter_sequence()),) and ((pp_tokens := self.pp_tokens()),) and (new_line := self.new_line()):
            return Embed(location=a.location, filename=b.content, search_current_path=True, parameters=c)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and (c := self.identifier_list()) and self.expect(TokenKind.COMMA) and self.expect(TokenKind.ELLIPSIS) and self.expect(TokenKind.R_PAREN) and ((d := self.replacement_list()),) and (new_line := self.new_line()):
            return DefineDirective(location=a.location, name=b.text, parameters=c, hasvarparam=True, is_object_like=False, replacement=d)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and ((c := self.identifier_list()),) and self.expect(TokenKind.R_PAREN) and ((d := self.replacement_list()),) and (new_line := self.new_line()):
            return DefineDirective(location=a.location, name=b.text, parameters=c if c != None else [], hasvarparam=False, is_object_like=False, replacement=d)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and self.expect(TokenKind.ELLIPSIS) and self.expect(TokenKind.R_PAREN) and ((d := self.replacement_list()),) and (new_line := self.new_line()):
            return DefineDirective(location=a.location, name=b.text, parameters=[], hasvarparam=True, is_object_like=False, replacement=d)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and ((d := self.replacement_list()),) and (new_line := self.new_line()):
            return DefineDirective(location=a.location, name=b.text, parameters=[], hasvarparam=False, is_object_like=True, replacement=d)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.UNDEF)) and (b := self.identifier()) and (new_line := self.new_line()):
            return UndefDirective(location=a.location, name=b.text)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.LINE)) and (b := self.integer_constant()) and ((c := self.string_literal()),) and (new_line := self.new_line()):
            return LineDirecvtive(location=a.location, lineno=int(b.text), filename=c if c != None else '')
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ERROR)) and ((b := self.pp_tokens()),) and (new_line := self.new_line()):
            return ErrorDirecvtive(location=a.location, messages=b if b != None else [])
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.WARNING)) and ((b := self.pp_tokens()),) and (new_line := self.new_line()):
            return WarningDirecvtive(location=a.location, messages=b if b != None else [])
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.PRAGMA)) and ((b := self.pp_tokens()),) and (new_line := self.new_line()):
            return Pragma(location=a.location, args=b if b != None else [])
        self.restore(_z)
        if (a := self.expect(TokenKind.HASH)) and (new_line := self.new_line()):
            return EmptyDirective(location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and self.expect(TokenKind.INCLUDE) and (header_name := self.header_name()) and ((pp_tokens := self.pp_tokens()),):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and self.expect(TokenKind.INCLUDE) and (string_literal := self.string_literal()) and ((pp_tokens := self.pp_tokens()),):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and self.expect(TokenKind.INCLUDE):
            return self.error('期待"文件名"或<文件名>', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and self.expect(TokenKind.EMBED) and (header_name := self.header_name()) and ((embed_parameter_sequence := self.embed_parameter_sequence()),) and ((pp_tokens := self.pp_tokens()),):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and self.expect(TokenKind.EMBED) and (string_literal := self.string_literal()) and ((embed_parameter_sequence := self.embed_parameter_sequence()),) and ((pp_tokens := self.pp_tokens()),):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and self.expect(TokenKind.EMBED):
            return self.error('期待"文件名"或<文件名>', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and (c := self.identifier_list()) and self.expect(TokenKind.COMMA) and self.expect(TokenKind.ELLIPSIS) and self.expect(TokenKind.R_PAREN) and (d := self.replacement_list()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and (c := self.identifier_list()) and self.expect(TokenKind.COMMA) and self.expect(TokenKind.ELLIPSIS) and self.expect(TokenKind.R_PAREN):
            return self.error('期待替换列表', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and (c := self.identifier_list()) and self.expect(TokenKind.COMMA) and self.expect(TokenKind.ELLIPSIS):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and (c := self.identifier_list()) and self.expect(TokenKind.COMMA):
            return self.error("期待'...'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and ((c := self.identifier_list()),) and self.expect(TokenKind.R_PAREN) and (d := self.replacement_list()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and ((c := self.identifier_list()),) and self.expect(TokenKind.R_PAREN):
            return self.error('期待替换列表', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and ((c := self.identifier_list()),):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and self.expect(TokenKind.ELLIPSIS) and self.expect(TokenKind.R_PAREN) and (d := self.replacement_list()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and self.expect(TokenKind.ELLIPSIS) and self.expect(TokenKind.R_PAREN):
            return self.error('期待替换列表', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (lparen := self.lparen()) and self.expect(TokenKind.ELLIPSIS):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()) and (d := self.replacement_list()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)) and (b := self.identifier()):
            return self.error('期待替换列表', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.DEFINE)):
            return self.error('期待标识符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.UNDEF)) and (b := self.identifier()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and self.expect(TokenKind.UNDEF):
            return self.error('期待标识符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.LINE)) and (b := self.integer_constant()) and ((c := self.string_literal()),):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and self.expect(TokenKind.LINE):
            return self.error('期待整数', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ERROR)) and ((b := self.pp_tokens()),):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.WARNING)) and ((b := self.pp_tokens()),):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.PRAGMA)) and ((b := self.pp_tokens()),):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def identifier_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.identifier_list()) and self.expect(TokenKind.COMMA) and (b := self.identifier()):
            return a + [b.text]
        self.restore(_z)
        if (a := self.identifier()):
            return [a.text]
        self.restore(_z)
        return None

    @memorize
    def replacement_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.pp_tokens()),):
            return a if a != None else []
        self.restore(_z)
        return None

    @memorize
    def if_section(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.if_group()) and ((b := self.elif_groups()),) and ((c := self.else_group()),) and (endif_line := self.endif_line()):
            return reconstruct(a, (b if b != None else []) + ([c] if c != None else []))
        self.restore(_z)
        if (a := self.if_group()) and ((b := self.elif_groups()),) and ((c := self.else_group()),):
            return self.error('缺少#endif', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def if_group(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IF)) and (b := self.constant_expression()) and (new_line := self.new_line()) and ((c := self.group()),):
            return IfDirecvtive(location=a.location, symtab=self.tokengen.symtab, expr=b, group=c if c != None else [])
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IFDEF)) and (b := self.identifier()) and (new_line := self.new_line()) and ((c := self.group()),):
            return IfdefDirecvtive(location=a.location, symtab=self.tokengen.symtab, name=b.text, group=c if c != None else [])
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IFNDEF)) and (b := self.identifier()) and (new_line := self.new_line()) and ((c := self.group()),):
            return IfndefDirecvtive(location=a.location, symtab=self.tokengen.symtab, name=b.text, group=c if c != None else [])
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IF)) and (b := self.constant_expression()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IF)):
            return self.error('期待表达式', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IFDEF)) and (b := self.identifier()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IFDEF)):
            return self.error('期待标识符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IFNDEF)) and (b := self.identifier()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.IFNDEF)):
            return self.error('期待标识符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIF)) and (b := self.constant_expression()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIF)):
            return self.error('期待表达式', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIFDEF)) and (b := self.identifier()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIFDEF)):
            return self.error('期待标识符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIFNDEF)) and (b := self.identifier()):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIFNDEF)):
            return self.error('期待标识符', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def elif_groups(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.elif_groups()) and (b := self.elif_group()):
            return a + [b]
        self.restore(_z)
        if (b := self.elif_group()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def elif_group(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIF)) and (b := self.constant_expression()) and (new_line := self.new_line()) and ((c := self.group()),):
            return ElifDirecvtive(location=a.location, symtab=self.tokengen.symtab, expr=b, group=c if c != None else [])
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIFDEF)) and (b := self.identifier()) and (new_line := self.new_line()) and ((c := self.group()),):
            return IfdefDirecvtive(location=a.location, symtab=self.tokengen.symtab, name=b.text, group=c if c != None else [])
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELIFNDEF)) and (b := self.identifier()) and (new_line := self.new_line()) and ((c := self.group()),):
            return IfndefDirecvtive(location=a.location, symtab=self.tokengen.symtab, name=b.text, group=c if c != None else [])
        self.restore(_z)
        return None

    @memorize
    def else_group(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELSE)) and (new_line := self.new_line()) and ((b := self.group()),):
            return ElseDirecvtive(location=a.location, group=b if b != None else [])
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ELSE)):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def endif_line(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ENDIF)) and (new_line := self.new_line()):
            return EndifDirecvtive(location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.HASH) and (a := self.expect(TokenKind.ENDIF)):
            return self.error('缺少换行符', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def group_part(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (if_section := self.if_section()):
            return if_section
        self.restore(_z)
        if (control_line := self.control_line()):
            return control_line
        self.restore(_z)
        if ((text_line := self.text_line()),):
            return text_line
        self.restore(_z)
        return None

    @memorize
    def text_line(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.pp_tokens()),) and (new_line := self.new_line()):
            return a if a != None else []
        self.restore(_z)
        return None

    @memorize
    def pp_parameter_name(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (pp_prefixed_parameter := self.pp_prefixed_parameter()):
            return pp_prefixed_parameter
        self.restore(_z)
        if (pp_standard_parameter := self.pp_standard_parameter()):
            return pp_standard_parameter
        self.restore(_z)
        return None

    @memorize
    def pp_standard_parameter(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.identifier()):
            return ('', a.text)
        self.restore(_z)
        return None

    @memorize
    def pp_prefixed_parameter(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.identifier()) and self.expect(TokenKind.COLONCOLON) and (b := self.identifier()):
            return (a.text, b.text)
        self.restore(_z)
        if (a := self.identifier()) and self.expect(TokenKind.COLONCOLON):
            return self.error('期待另一个标识符', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def limit_name(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='limit')):
            return a
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='__limit__')):
            return a
        self.restore(_z)
        return None

    @memorize
    def other_standard_name(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='suffix')):
            return (a, SuffixParam)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='__suffix__')):
            return (a, SuffixParam)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='prefix')):
            return (a, PrefixParam)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='__prefix__')):
            return (a, PrefixParam)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='if_empty')):
            return (a, IfEmptyParam)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='__if_empty__')):
            return (a, IfEmptyParam)
        self.restore(_z)
        return None

    @memorize
    def pp_parameter(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.limit_name()) and self.expect(TokenKind.L_PAREN) and (b := self.constant_expression()) and self.expect(TokenKind.R_PAREN):
            return LimitParam(location=a.location, expr=b)
        self.restore(_z)
        if (a := self.other_standard_name()) and ((b := self.pp_parameter_clause()),):
            return a[1](location=a[0].location, args=b if b != None else [])
        self.restore(_z)
        if (a := self.pp_parameter_name()) and ((b := self.pp_parameter_clause()),):
            return PPParameter(location=begin_location, prefix_name=a[0], name=a[1], args=b if b != None else [])
        self.restore(_z)
        if (a := self.limit_name()) and self.expect(TokenKind.L_PAREN) and (b := self.constant_expression()):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if (a := self.limit_name()) and self.expect(TokenKind.L_PAREN):
            return self.error('期待表达式', self.curtoken().location)
        self.restore(_z)
        if (a := self.limit_name()):
            return self.error("期待'('", self.curtoken().location)
        self.restore(_z)
        if (a := self.other_standard_name()) and self.expect(TokenKind.L_PAREN) and ((a := self.pp_balanced_token_sequence()),):
            return self.error("期待')'", self.curtoken().location)
        self.restore(_z)
        if (a := self.other_standard_name()):
            return self.error("期待'('", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def pp_parameter_clause(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if self.expect(TokenKind.L_PAREN) and ((a := self.pp_balanced_token_sequence()),) and self.expect(TokenKind.R_PAREN):
            return a if a != None else []
        self.restore(_z)
        return None

    @memorize_left_rec
    def pp_balanced_token_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.pp_balanced_token_sequence()) and (b := self.pp_balanced_token()):
            return a + b
        self.restore(_z)
        if (b := self.pp_balanced_token()):
            return b
        self.restore(_z)
        return None

    @memorize_left_rec
    def embed_parameter_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.embed_parameter_sequence()) and (b := self.pp_parameter()):
            return a + [b]
        self.restore(_z)
        if (b := self.pp_parameter()):
            return [b]
        self.restore(_z)
        return None