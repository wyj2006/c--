from parse import ParserBase, memorize, memorize_left_rec
from basic import *
from cast import *

def assign(a, attr, b):
    setattr(a, attr, b)
    return a

def option(a, default):
    return a if a != None else default

def concat_pointer(a, b):
    if a == None:
        return b
    c = a
    while c.declarator != None:
        c = c.declarator
    c.declarator = b
    return a

class Gen_CParser(ParserBase):

    @memorize
    def start(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.translation_unit()) and (end := self.end()):
            return TranslationUnit(body=a, location=begin_location)
        self.restore(_z)
        if (translation_unit := self.translation_unit()):
            return self.error('解析已结束, 但文件未结束', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def translation_unit(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.translation_unit()) and (b := self.external_declaration()):
            return a + [b]
        self.restore(_z)
        if (b := self.external_declaration()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def external_declaration(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (function_definition := self.function_definition()):
            return function_definition
        self.restore(_z)
        if (declaration := self.declaration()):
            return declaration
        self.restore(_z)
        return None

    @memorize
    def function_definition(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.attribute_specifier_sequence()),) and (b := self.declaration_specifiers()) and (c := self.declarator()) and (d := self.function_body()):
            return FunctionDef(attribute_specifiers=option(a, []), specifiers=b[0], specifier_attributes=b[1], declarator=c, body=d, location=begin_location)
        self.restore(_z)
        return None

    @memorize
    def function_body(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (compound_statement := self.compound_statement()):
            return compound_statement
        self.restore(_z)
        return None

    @memorize
    def statement(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (labeled_statement := self.labeled_statement()):
            return labeled_statement
        self.restore(_z)
        if (unlabeled_statement := self.unlabeled_statement()):
            return unlabeled_statement
        self.restore(_z)
        return None

    @memorize
    def unlabeled_statement(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (expression_statement := self.expression_statement()):
            return expression_statement
        self.restore(_z)
        if ((b := self.attribute_specifier_sequence()),) and (a := self.primary_block()):
            return assign(a, 'attribute_specifiers', option(b, []))
        self.restore(_z)
        if ((b := self.attribute_specifier_sequence()),) and (a := self.jump_statement()):
            return assign(a, 'attribute_specifiers', option(b, []))
        self.restore(_z)
        return None

    @memorize
    def primary_block(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (compound_statement := self.compound_statement()):
            return compound_statement
        self.restore(_z)
        if (selection_statement := self.selection_statement()):
            return selection_statement
        self.restore(_z)
        if (iteration_statement := self.iteration_statement()):
            return iteration_statement
        self.restore(_z)
        return None

    @memorize
    def secondary_block(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (statement := self.statement()):
            return statement
        self.restore(_z)
        return None

    @memorize
    def label(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((b := self.attribute_specifier_sequence()),) and (a := self.identifier()) and self.expect(TokenKind.COLON):
            return LabelStmt(name=a.text, attribute_specifiers=option(b, []), location=a.location)
        self.restore(_z)
        if ((b := self.attribute_specifier_sequence()),) and (a := self.expect(TokenKind.CASE)) and (c := self.constant_expression()) and self.expect(TokenKind.COLON):
            return CaseStmt(expr=c, attribute_specifiers=option(b, []), location=a.location)
        self.restore(_z)
        if ((b := self.attribute_specifier_sequence()),) and (a := self.expect(TokenKind.DEFAULT)) and self.expect(TokenKind.COLON):
            return DefaultStmt(attribute_specifiers=option(b, []), location=a.location)
        self.restore(_z)
        if ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and (a := self.expect(TokenKind.CASE)) and (constant_expression := self.constant_expression()):
            return self.error("'case'后的常量表达式后应有':'", a.location)
        self.restore(_z)
        if ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and (a := self.expect(TokenKind.CASE)):
            return self.error("'case'后应跟常量表达式", a.location)
        self.restore(_z)
        if ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and (a := self.expect(TokenKind.DEFAULT)):
            return self.error("'default'后缺':'", a.location)
        self.restore(_z)
        return None

    @memorize
    def labeled_statement(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.label()) and (b := self.statement()):
            return assign(a, 'stmt', b)
        self.restore(_z)
        return None

    @memorize
    def compound_statement(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (b := self.expect(TokenKind.L_BRACE)) and ((a := self.block_item_list()),) and self.expect(TokenKind.R_BRACE):
            return CompoundStmt(items=option(a, []), location=b.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.L_BRACE)) and ((block_item_list := self.block_item_list()),):
            return self.error('大括号未闭合', a.location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def block_item_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.block_item_list()) and (b := self.block_item()):
            return a + [b]
        self.restore(_z)
        if (b := self.block_item()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def block_item(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (declaration := self.declaration()):
            return declaration
        self.restore(_z)
        if (unlabeled_statement := self.unlabeled_statement()):
            return unlabeled_statement
        self.restore(_z)
        if (label := self.label()):
            return label
        self.restore(_z)
        return None

    @memorize
    def expression_statement(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.attribute_specifier_sequence()) and (b := self.expression()) and self.expect(TokenKind.SEMI):
            return ExpressionStmt(attribute_specifiers=a, expr=b, location=a.location)
        self.restore(_z)
        if ((a := self.expression()),) and self.expect(TokenKind.SEMI):
            return ExpressionStmt(expr=a, location=begin_location)
        self.restore(_z)
        return None

    @memorize
    def selection_statement(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IF)) and self.expect(TokenKind.L_PAREN) and (b := self.expression()) and self.expect(TokenKind.R_PAREN) and (c := self.secondary_block()) and self.expect(TokenKind.ELSE) and (d := self.secondary_block()):
            return IfStmt(condition_expr=b, body=c, else_body=d, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.IF)) and self.expect(TokenKind.L_PAREN) and (b := self.expression()) and self.expect(TokenKind.R_PAREN) and (c := self.secondary_block()):
            return IfStmt(condition_expr=b, body=c, else_body=None, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.SWITCH)) and self.expect(TokenKind.L_PAREN) and (b := self.expression()) and self.expect(TokenKind.R_PAREN) and (c := self.secondary_block()):
            return SwitchStmt(condition_expr=b, body=c, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.IF) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()) and (a := self.expect(TokenKind.R_PAREN)) and (secondary_block := self.secondary_block()) and self.expect(TokenKind.ELSE):
            return self.error('缺少else的语句体', a.location)
        self.restore(_z)
        if self.expect(TokenKind.IF) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()) and (a := self.expect(TokenKind.R_PAREN)):
            return self.error('后面缺少语句体', a.location)
        self.restore(_z)
        if self.expect(TokenKind.IF) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()):
            return self.error("'('未闭合", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.IF) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个表达式', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.IF)):
            return self.error("后面应该为'('", a.location)
        self.restore(_z)
        if self.expect(TokenKind.SWITCH) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()) and (a := self.expect(TokenKind.R_PAREN)):
            return self.error('后面缺少语句体', a.location)
        self.restore(_z)
        if self.expect(TokenKind.SWITCH) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()):
            return self.error("'('未闭合", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.SWITCH) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个表达式', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.SWITCH)):
            return self.error("后面应该为'('", a.location)
        self.restore(_z)
        return None

    @memorize
    def iteration_statement(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.WHILE)) and self.expect(TokenKind.L_PAREN) and (b := self.expression()) and self.expect(TokenKind.R_PAREN) and (c := self.secondary_block()):
            return WhileStmt(condition_expr=b, body=c, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.DO)) and (b := self.secondary_block()) and self.expect(TokenKind.WHILE) and self.expect(TokenKind.L_PAREN) and (c := self.expression()) and self.expect(TokenKind.R_PAREN) and self.expect(TokenKind.SEMI):
            return DoWhileStmt(condition_expr=c, body=b, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.FOR)) and self.expect(TokenKind.L_PAREN) and ((b := self.expression()),) and self.expect(TokenKind.SEMI) and ((c := self.expression()),) and self.expect(TokenKind.SEMI) and ((d := self.expression()),) and self.expect(TokenKind.R_PAREN) and (e := self.secondary_block()):
            return ForStmt(init=b, condition_expr=c, increase_expr=d, body=e, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.FOR)) and self.expect(TokenKind.L_PAREN) and (b := self.declaration()) and ((c := self.expression()),) and self.expect(TokenKind.SEMI) and ((d := self.expression()),) and self.expect(TokenKind.R_PAREN) and (e := self.secondary_block()):
            return ForStmt(init=b, condition_expr=c, increase_expr=d, body=e, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.WHILE) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()) and (a := self.expect(TokenKind.R_PAREN)):
            return self.error('后面缺少语句体', a.location)
        self.restore(_z)
        if self.expect(TokenKind.WHILE) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()):
            return self.error("'('未闭合", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.WHILE) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个表达式', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.WHILE)):
            return self.error("后面应该为'('", a.location)
        self.restore(_z)
        if self.expect(TokenKind.DO) and (secondary_block := self.secondary_block()) and self.expect(TokenKind.WHILE) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()) and (a := self.expect(TokenKind.R_PAREN)):
            return self.error("do-while语句后应有';'", a.location)
        self.restore(_z)
        if self.expect(TokenKind.DO) and (secondary_block := self.secondary_block()) and self.expect(TokenKind.WHILE) and self.expect(TokenKind.L_PAREN) and (expression := self.expression()):
            return self.error("'('未闭合", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.DO) and (secondary_block := self.secondary_block()) and self.expect(TokenKind.WHILE) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个表达式', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.DO)) and (secondary_block := self.secondary_block()) and self.expect(TokenKind.WHILE):
            return self.error("后面应该为'('", a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.DO)):
            return self.error("'do'后面缺少语句体", a.location)
        self.restore(_z)
        if self.expect(TokenKind.FOR) and self.expect(TokenKind.L_PAREN) and ((expression := self.expression()),) and self.expect(TokenKind.SEMI) and ((expression := self.expression()),) and self.expect(TokenKind.SEMI) and ((expression := self.expression()),) and (a := self.expect(TokenKind.R_PAREN)):
            return self.error('缺少for的语句体', a.location)
        self.restore(_z)
        if self.expect(TokenKind.FOR) and self.expect(TokenKind.L_PAREN) and ((expression := self.expression()),) and self.expect(TokenKind.SEMI) and ((expression := self.expression()),) and self.expect(TokenKind.SEMI) and ((expression := self.expression()),):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.FOR) and self.expect(TokenKind.L_PAREN) and ((expression := self.expression()),) and self.expect(TokenKind.SEMI) and ((expression := self.expression()),):
            return self.error("前面应有';'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.FOR) and self.expect(TokenKind.L_PAREN) and ((expression := self.expression()),):
            return self.error("前面应有';'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.FOR) and self.expect(TokenKind.L_PAREN) and (declaration := self.declaration()) and ((expression := self.expression()),) and self.expect(TokenKind.SEMI) and ((expression := self.expression()),) and (a := self.expect(TokenKind.R_PAREN)):
            return self.error('缺少for的语句体', a.location)
        self.restore(_z)
        if self.expect(TokenKind.FOR) and self.expect(TokenKind.L_PAREN) and (declaration := self.declaration()) and ((expression := self.expression()),) and self.expect(TokenKind.SEMI) and ((expression := self.expression()),):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.FOR) and self.expect(TokenKind.L_PAREN) and (declaration := self.declaration()) and ((expression := self.expression()),):
            return self.error("前面应有';'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.FOR) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待表达式或声明', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.FOR)):
            return self.error("后面应该为'('", a.location)
        self.restore(_z)
        return None

    @memorize
    def jump_statement(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.GOTO)) and (b := self.identifier()) and self.expect(TokenKind.SEMI):
            return GotoStmt(name=b.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.CONTINUE)) and self.expect(TokenKind.SEMI):
            return ContinueStmt(location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.BREAK)) and self.expect(TokenKind.SEMI):
            return BreakStmt(location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.RETURN)) and ((b := self.expression()),) and self.expect(TokenKind.SEMI):
            return ReturnStmt(expr=b, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.GOTO) and (identifier := self.identifier()):
            return self.error("前面期待一个';'", self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.GOTO)):
            return self.error('goto后面应有一个标识符', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.CONTINUE)):
            return self.error("后面缺少一个';'", a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.BREAK)):
            return self.error("后面缺少一个';'", a.location)
        self.restore(_z)
        if self.expect(TokenKind.RETURN) and ((expression := self.expression()),):
            return self.error("前面缺少一个';'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def constant_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (conditional_expression := self.conditional_expression()):
            return conditional_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expression()) and (c := self.expect(TokenKind.COMMA)) and (b := self.assignment_expression()):
            return BinaryOperator(op=BinOpKind.COMMA, left=a, right=b, location=c.location)
        self.restore(_z)
        if (assignment_expression := self.assignment_expression()):
            return assignment_expression
        self.restore(_z)
        return None

    @memorize
    def assignment_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.unary_expression()) and (c := self.assignment_operator()) and (b := self.assignment_expression()):
            return BinaryOperator(op=c[1], left=a, right=b, location=c[0].location)
        self.restore(_z)
        if (conditional_expression := self.conditional_expression()):
            return conditional_expression
        self.restore(_z)
        return None

    @memorize
    def assignment_operator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.EQUAL)):
            return (a, BinOpKind.ASSIGN)
        self.restore(_z)
        if (a := self.expect(TokenKind.STAREQUAL)):
            return (a, BinOpKind.AMUL)
        self.restore(_z)
        if (a := self.expect(TokenKind.SLASHEQUAL)):
            return (a, BinOpKind.ADIV)
        self.restore(_z)
        if (a := self.expect(TokenKind.PERCENTEQUAL)):
            return (a, BinOpKind.AMOD)
        self.restore(_z)
        if (a := self.expect(TokenKind.PLUSEQUAL)):
            return (a, BinOpKind.AADD)
        self.restore(_z)
        if (a := self.expect(TokenKind.MINUSEQUAL)):
            return (a, BinOpKind.ASUB)
        self.restore(_z)
        if (a := self.expect(TokenKind.LESSLESSEQUAL)):
            return (a, BinOpKind.ALSHIFT)
        self.restore(_z)
        if (a := self.expect(TokenKind.GREATERGREATEREQUAL)):
            return (a, BinOpKind.ARSHIFT)
        self.restore(_z)
        if (a := self.expect(TokenKind.AMPEQUAL)):
            return (a, BinOpKind.ABITAND)
        self.restore(_z)
        if (a := self.expect(TokenKind.CARETEQUAL)):
            return (a, BinOpKind.ABITXOR)
        self.restore(_z)
        if (a := self.expect(TokenKind.PIPEEQUAL)):
            return (a, BinOpKind.ABITOR)
        self.restore(_z)
        return None

    @memorize
    def conditional_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.logical_OR_expression()) and (d := self.expect(TokenKind.QUESTION)) and (b := self.expression()) and self.expect(TokenKind.COLON) and (c := self.conditional_expression()):
            return ConditionalOperator(condition_expr=a, true_expr=b, false_expr=c, location=d.location)
        self.restore(_z)
        if (logical_OR_expression := self.logical_OR_expression()):
            return logical_OR_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def logical_OR_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.logical_OR_expression()) and (c := self.expect(TokenKind.PIPEPIPE)) and (b := self.logical_AND_expression()):
            return BinaryOperator(op=BinOpKind.OR, left=a, right=b, location=c.location)
        self.restore(_z)
        if (logical_AND_expression := self.logical_AND_expression()):
            return logical_AND_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def logical_AND_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.logical_AND_expression()) and (c := self.expect(TokenKind.AMPAMP)) and (b := self.inclusive_OR_expression()):
            return BinaryOperator(op=BinOpKind.AND, left=a, right=b, location=c.location)
        self.restore(_z)
        if (inclusive_OR_expression := self.inclusive_OR_expression()):
            return inclusive_OR_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def inclusive_OR_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.inclusive_OR_expression()) and (c := self.expect(TokenKind.PIPE)) and (b := self.exclusive_OR_expression()):
            return BinaryOperator(op=BinOpKind.BITOR, left=a, right=b, location=c.location)
        self.restore(_z)
        if (exclusive_OR_expression := self.exclusive_OR_expression()):
            return exclusive_OR_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def exclusive_OR_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.exclusive_OR_expression()) and (c := self.expect(TokenKind.CARET)) and (b := self.AND_expression()):
            return BinaryOperator(op=BinOpKind.BITXOR, left=a, right=b, location=c.location)
        self.restore(_z)
        if (AND_expression := self.AND_expression()):
            return AND_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def AND_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.AND_expression()) and (c := self.expect(TokenKind.AMP)) and (b := self.equality_expression()):
            return BinaryOperator(op=BinOpKind.BITAND, left=a, right=b, location=c.location)
        self.restore(_z)
        if (equality_expression := self.equality_expression()):
            return equality_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def equality_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.equality_expression()) and (c := self.expect(TokenKind.EQUALEQUAL)) and (b := self.relational_expression()):
            return BinaryOperator(op=BinOpKind.EQ, left=a, right=b, location=c.location)
        self.restore(_z)
        if (a := self.equality_expression()) and (c := self.expect(TokenKind.EXCLAIMEQUAL)) and (b := self.relational_expression()):
            return BinaryOperator(op=BinOpKind.NEQ, left=a, right=b, location=c.location)
        self.restore(_z)
        if (relational_expression := self.relational_expression()):
            return relational_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def relational_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.relational_expression()) and (c := self.expect(TokenKind.LESS)) and (b := self.shift_expression()):
            return BinaryOperator(op=BinOpKind.LT, left=a, right=b, location=c.location)
        self.restore(_z)
        if (a := self.relational_expression()) and (c := self.expect(TokenKind.GREATER)) and (b := self.shift_expression()):
            return BinaryOperator(op=BinOpKind.GT, left=a, right=b, location=c.location)
        self.restore(_z)
        if (a := self.relational_expression()) and (c := self.expect(TokenKind.LESSEQUAL)) and (b := self.shift_expression()):
            return BinaryOperator(op=BinOpKind.LTE, left=a, right=b, location=c.location)
        self.restore(_z)
        if (a := self.relational_expression()) and (c := self.expect(TokenKind.GREATEREQUAL)) and (b := self.shift_expression()):
            return BinaryOperator(op=BinOpKind.GTE, left=a, right=b, location=c.location)
        self.restore(_z)
        if (shift_expression := self.shift_expression()):
            return shift_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def shift_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.shift_expression()) and (c := self.expect(TokenKind.LESSLESS)) and (b := self.additive_expression()):
            return BinaryOperator(op=BinOpKind.LSHIFT, left=a, right=b, location=c.location)
        self.restore(_z)
        if (a := self.shift_expression()) and (c := self.expect(TokenKind.GREATERGREATER)) and (b := self.additive_expression()):
            return BinaryOperator(op=BinOpKind.RSHIFT, left=a, right=b, location=c.location)
        self.restore(_z)
        if (additive_expression := self.additive_expression()):
            return additive_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def additive_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.additive_expression()) and (c := self.expect(TokenKind.PLUS)) and (b := self.multiplicative_expression()):
            return BinaryOperator(op=BinOpKind.ADD, left=a, right=b, location=c.location)
        self.restore(_z)
        if (a := self.additive_expression()) and (c := self.expect(TokenKind.MINUS)) and (b := self.multiplicative_expression()):
            return BinaryOperator(op=BinOpKind.SUB, left=a, right=b, location=c.location)
        self.restore(_z)
        if (multiplicative_expression := self.multiplicative_expression()):
            return multiplicative_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def multiplicative_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.multiplicative_expression()) and (c := self.expect(TokenKind.STAR)) and (b := self.cast_expression()):
            return BinaryOperator(op=BinOpKind.MUL, left=a, right=b, location=c.location)
        self.restore(_z)
        if (a := self.multiplicative_expression()) and (c := self.expect(TokenKind.SLASH)) and (b := self.cast_expression()):
            return BinaryOperator(op=BinOpKind.DIV, left=a, right=b, location=c.location)
        self.restore(_z)
        if (a := self.multiplicative_expression()) and (c := self.expect(TokenKind.PERCENT)) and (b := self.cast_expression()):
            return BinaryOperator(op=BinOpKind.MOD, left=a, right=b, location=c.location)
        self.restore(_z)
        if (cast_expression := self.cast_expression()):
            return cast_expression
        self.restore(_z)
        return None

    @memorize
    def cast_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.L_PAREN)) and (b := self.type_name()) and self.expect(TokenKind.R_PAREN) and (c := self.cast_expression()):
            return ExplicitCast(type_name=b, expr=c, location=a.location)
        self.restore(_z)
        if (unary_expression := self.unary_expression()):
            return unary_expression
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and (type_name := self.type_name()):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def unary_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.PLUSPLUS)) and (b := self.unary_expression()):
            return UnaryOperator(op=UnaryOpKind.PREFIX_INC, operand=b, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.MINUSMINUS)) and (b := self.unary_expression()):
            return UnaryOperator(op=UnaryOpKind.PREFIX_DEC, operand=b, location=a.location)
        self.restore(_z)
        if (a := self.unary_operator()) and (b := self.cast_expression()):
            return UnaryOperator(op=a[1], operand=b, location=a[0].location)
        self.restore(_z)
        if (a := self.expect(TokenKind.SIZEOF)) and (b := self.unary_expression()):
            return UnaryOperator(op=UnaryOpKind.SIZEOF, operand=b, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.SIZEOF)) and self.expect(TokenKind.L_PAREN) and (b := self.type_name()) and self.expect(TokenKind.R_PAREN):
            return UnaryOperator(op=UnaryOpKind.SIZEOF, operand=b, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.ALIGNOF)) and self.expect(TokenKind.L_PAREN) and (b := self.type_name()) and self.expect(TokenKind.R_PAREN):
            return UnaryOperator(op=UnaryOpKind.ALIGNOF, operand=b, location=a.location)
        self.restore(_z)
        if (postfix_expression := self.postfix_expression()):
            return postfix_expression
        self.restore(_z)
        if self.expect(TokenKind.SIZEOF) and self.expect(TokenKind.L_PAREN) and (type_name := self.type_name()):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.ALIGNOF) and self.expect(TokenKind.L_PAREN) and (type_name := self.type_name()):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def unary_operator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.AMP)):
            return (a, UnaryOpKind.ADDRESS)
        self.restore(_z)
        if (a := self.expect(TokenKind.STAR)):
            return (a, UnaryOpKind.DEREFERENCE)
        self.restore(_z)
        if (a := self.expect(TokenKind.PLUS)):
            return (a, UnaryOpKind.POSITIVE)
        self.restore(_z)
        if (a := self.expect(TokenKind.MINUS)):
            return (a, UnaryOpKind.NEGATIVE)
        self.restore(_z)
        if (a := self.expect(TokenKind.TILDE)):
            return (a, UnaryOpKind.INVERT)
        self.restore(_z)
        if (a := self.expect(TokenKind.EXCLAIM)):
            return (a, UnaryOpKind.NOT)
        self.restore(_z)
        return None

    @memorize_left_rec
    def postfix_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.postfix_expression()) and (c := self.expect(TokenKind.L_SQUARE)) and (b := self.expression()) and self.expect(TokenKind.R_SQUARE):
            return ArraySubscript(array=a, index=b, location=c.location)
        self.restore(_z)
        if (a := self.postfix_expression()) and (c := self.expect(TokenKind.L_PAREN)) and ((b := self.argument_expression_list()),) and self.expect(TokenKind.R_PAREN):
            return FunctionCall(func=a, args=option(b, []), location=c.location)
        self.restore(_z)
        if (a := self.postfix_expression()) and (c := self.expect(TokenKind.PERIOD)) and (b := self.identifier()):
            return MemberRef(target=a, member_name=b.text, is_arrow=False, location=c.location)
        self.restore(_z)
        if (a := self.postfix_expression()) and (c := self.expect(TokenKind.ARROW)) and (b := self.identifier()):
            return MemberRef(target=a, member_name=b.text, is_arrow=True, location=c.location)
        self.restore(_z)
        if (a := self.postfix_expression()) and (b := self.expect(TokenKind.PLUSPLUS)):
            return UnaryOperator(op=UnaryOpKind.POSTFIX_INC, operand=a, location=b.location)
        self.restore(_z)
        if (a := self.postfix_expression()) and (b := self.expect(TokenKind.MINUSMINUS)):
            return UnaryOperator(op=UnaryOpKind.POSTFIX_DEC, operand=a, location=b.location)
        self.restore(_z)
        if (compound_literal := self.compound_literal()):
            return compound_literal
        self.restore(_z)
        if (primary_expression := self.primary_expression()):
            return primary_expression
        self.restore(_z)
        return None

    @memorize_left_rec
    def argument_expression_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.argument_expression_list()) and self.expect(TokenKind.COMMA) and (b := self.assignment_expression()):
            return a + [b]
        self.restore(_z)
        if (b := self.assignment_expression()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def compound_literal(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.L_PAREN)) and ((b := self.storage_class_specifiers()),) and (c := self.type_name()) and self.expect(TokenKind.R_PAREN) and (d := self.braced_initializer()):
            return CompoundLiteral(storage_class=option(b, []), type_name=c, initializer=d, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and (storage_class_specifiers := self.storage_class_specifiers()) and (type_name := self.type_name()):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def storage_class_specifiers(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.storage_class_specifiers()) and (b := self.storage_class_specifier()):
            return a + [b]
        self.restore(_z)
        if (b := self.storage_class_specifier()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def generic_selection(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind._GENERIC)) and self.expect(TokenKind.L_PAREN) and (b := self.assignment_expression()) and self.expect(TokenKind.COMMA) and (c := self.generic_assoc_list()) and self.expect(TokenKind.R_PAREN):
            return GenericSelection(controling_expr=b, assoc_list=c, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind._GENERIC) and self.expect(TokenKind.L_PAREN) and (assignment_expression := self.assignment_expression()) and self.expect(TokenKind.COMMA) and (generic_assoc_list := self.generic_assoc_list()):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind._GENERIC) and self.expect(TokenKind.L_PAREN) and (assignment_expression := self.assignment_expression()) and (a := self.expect(TokenKind.COMMA)):
            return self.error('后面缺少关联列表', a.location)
        self.restore(_z)
        if self.expect(TokenKind._GENERIC) and self.expect(TokenKind.L_PAREN) and (assignment_expression := self.assignment_expression()):
            return self.error("前面应有','", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind._GENERIC) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面缺少一个控制表达式', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind._GENERIC)):
            return self.error("后面缺少'('", a.location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def generic_assoc_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.generic_assoc_list()) and self.expect(TokenKind.COMMA) and (b := self.generic_association()):
            return a + [b]
        self.restore(_z)
        if (b := self.generic_association()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def generic_association(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.type_name()) and self.expect(TokenKind.COLON) and (b := self.assignment_expression()):
            return GenericAssociation(type_name=a, expr=b, is_default=False, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.DEFAULT)) and self.expect(TokenKind.COLON) and (b := self.assignment_expression()):
            return GenericAssociation(expr=b, is_default=True, location=a.location)
        self.restore(_z)
        if (type_name := self.type_name()) and (a := self.expect(TokenKind.COLON)):
            return self.error('后面缺少一个表达式', a.location)
        self.restore(_z)
        if (type_name := self.type_name()):
            return self.error("前面缺少一个':'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.DEFAULT) and (a := self.expect(TokenKind.COLON)):
            return self.error('后面缺少一个表达式', a.location)
        self.restore(_z)
        if self.expect(TokenKind.DEFAULT):
            return self.error("前面缺少一个':'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def primary_expression(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.identifier()):
            return Reference(name=a.text, location=a.location)
        self.restore(_z)
        if (constant := self.constant()):
            return constant
        self.restore(_z)
        if (a := self.string_literal()):
            return StringLiteral(value=a.content + '\x00', prefix=a.prefix, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and (a := self.expression()) and self.expect(TokenKind.R_PAREN):
            return a
        self.restore(_z)
        if (generic_selection := self.generic_selection()):
            return generic_selection
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and (expression := self.expression()):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.integer_constant()):
            return IntegerLiteral(value=a.content, prefix=a.prefix, suffix=a.suffix, location=a.location)
        self.restore(_z)
        if (a := self.floating_constant()):
            return FloatLiteral(value=a.content, prefix=a.prefix, suffix=a.suffix, location=a.location)
        self.restore(_z)
        if (a := self.character_constant()):
            return CharLiteral(value=a.content, prefix=a.prefix, location=a.location)
        self.restore(_z)
        if (predefined_constant := self.predefined_constant()):
            return predefined_constant
        self.restore(_z)
        return None

    @memorize
    def predefined_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.TRUE)):
            return BoolLiteral(value=True, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.FALSE)):
            return BoolLiteral(value=False, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.NULLPTR)):
            return NullPtrLiteral(location=a.location)
        self.restore(_z)
        return None

    @memorize
    def declaration(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.attribute_specifier_sequence()) and (b := self.declaration_specifiers()) and (c := self.init_declarator_list()) and (d := self.expect(TokenKind.SEMI)):
            return DeclStmt(attribute_specifiers=a, specifiers=b[0], specifier_attributes=b[1], declarators=c, location=d.location)
        self.restore(_z)
        if (a := self.declaration_specifiers()) and ((b := self.init_declarator_list()),) and (c := self.expect(TokenKind.SEMI)):
            return DeclStmt(attribute_specifiers=[], specifiers=a[0], specifier_attributes=a[1], declarators=option(b, []), location=c.location)
        self.restore(_z)
        if (static_assert_declaration := self.static_assert_declaration()):
            return static_assert_declaration
        self.restore(_z)
        if (attribute_declaration := self.attribute_declaration()):
            return attribute_declaration
        self.restore(_z)
        if (attribute_specifier_sequence := self.attribute_specifier_sequence()) and (declaration_specifiers := self.declaration_specifiers()) and (init_declarator_list := self.init_declarator_list()):
            return self.error("前面缺少一个';'", self.curtoken().location)
        self.restore(_z)
        if (attribute_specifier_sequence := self.attribute_specifier_sequence()) and (declaration_specifiers := self.declaration_specifiers()):
            return self.error('如果有属性列表就应该有初始化列表', self.curtoken().location)
        self.restore(_z)
        if (declaration_specifiers := self.declaration_specifiers()) and ((init_declarator_list := self.init_declarator_list()),):
            return self.error("前面缺少一个';'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def declaration_specifiers(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.declaration_specifier()) and (b := self.declaration_specifiers()):
            return ([a] + b[0], b[1])
        self.restore(_z)
        if (a := self.declaration_specifier()) and ((b := self.attribute_specifier_sequence()),):
            return ([a], option(b, []))
        self.restore(_z)
        return None

    @memorize
    def declaration_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (storage_class_specifier := self.storage_class_specifier()):
            return storage_class_specifier
        self.restore(_z)
        if (type_specifier_qualifier := self.type_specifier_qualifier()):
            return type_specifier_qualifier
        self.restore(_z)
        if (function_specifier := self.function_specifier()):
            return function_specifier
        self.restore(_z)
        return None

    @memorize_left_rec
    def init_declarator_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.init_declarator_list()) and self.expect(TokenKind.COMMA) and (b := self.init_declarator()):
            return a + [b]
        self.restore(_z)
        if (b := self.init_declarator()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def init_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.declarator()) and (c := self.expect(TokenKind.EQUAL)) and (b := self.initializer()):
            return TypeOrVarDecl(declarator=a, initializer=b, location=c.location)
        self.restore(_z)
        if (a := self.declarator()):
            return TypeOrVarDecl(declarator=a, initializer=None, location=a.location)
        self.restore(_z)
        if (declarator := self.declarator()) and (a := self.expect(TokenKind.EQUAL)):
            return self.error('后面缺少初始化器', a.location)
        self.restore(_z)
        return None

    @memorize
    def attribute_declaration(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.attribute_specifier_sequence()) and (b := self.expect(TokenKind.SEMI)):
            return AttributeDeclStmt(attribute_specifiers=a, location=b.location)
        self.restore(_z)
        if (attribute_specifier_sequence := self.attribute_specifier_sequence()):
            return self.error("前面缺少一个';'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def storage_class_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.AUTO)):
            return StorageClass(specifier=StorageClassSpecifier.AUTO, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.CONSTEXPR)):
            return StorageClass(specifier=StorageClassSpecifier.CONSTEXPR, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.EXTERN)):
            return StorageClass(specifier=StorageClassSpecifier.EXTERN, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.REGISTER)):
            return StorageClass(specifier=StorageClassSpecifier.REGISTER, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.STATIC)):
            return StorageClass(specifier=StorageClassSpecifier.STATIC, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.THREAD_LOCAL)):
            return StorageClass(specifier=StorageClassSpecifier.THREAD_LOCAL, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.TYPEDEF)):
            return StorageClass(specifier=StorageClassSpecifier.TYPEDEF, location=a.location)
        self.restore(_z)
        return None

    @memorize
    def type_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (atomic_type_specifier := self.atomic_type_specifier()):
            return atomic_type_specifier
        self.restore(_z)
        if (struct_or_union_specifier := self.struct_or_union_specifier()):
            return struct_or_union_specifier
        self.restore(_z)
        if (enum_specifier := self.enum_specifier()):
            return enum_specifier
        self.restore(_z)
        if (a := self.typedef_name()):
            return TypedefSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (typeof_specifier := self.typeof_specifier()):
            return typeof_specifier
        self.restore(_z)
        if (a := self.expect(TokenKind._BITINT)) and self.expect(TokenKind.L_PAREN) and (b := self.constant_expression()) and self.expect(TokenKind.R_PAREN):
            return BitIntSpecifier(size=b, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.VOID)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.CHAR)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.SHORT)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.INT)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.LONG)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.FLOAT)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.DOUBLE)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.SIGNED)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.UNSIGNED)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.BOOL)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind._COMPLEX)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind._DECIMAL32)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind._DECIMAL64)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind._DECIMAL128)):
            return BasicTypeSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind._BITINT) and self.expect(TokenKind.L_PAREN) and (constant_expression := self.constant_expression()):
            return self.error("前面应有')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind._BITINT) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个常量表达式', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind._BITINT)):
            return self.error("后面缺少'('", a.location)
        self.restore(_z)
        return None

    @memorize
    def struct_or_union_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.struct_or_union()) and ((b := self.attribute_specifier_sequence()),) and ((c := self.identifier()),) and self.expect(TokenKind.L_BRACE) and (d := self.member_declaration_list()) and self.expect(TokenKind.R_BRACE):
            return RecordDecl(struct_or_union=a.text, attribute_specifiers=option(b, []), name=c.text if c != None else '', members_declaration=d, location=a.location)
        self.restore(_z)
        if (a := self.struct_or_union()) and ((b := self.attribute_specifier_sequence()),) and (c := self.identifier()):
            return RecordDecl(struct_or_union=a.text, attribute_specifiers=option(b, []), name=c.text, members_declaration=[], location=a.location)
        self.restore(_z)
        if (struct_or_union := self.struct_or_union()) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and ((identifier := self.identifier()),) and self.expect(TokenKind.L_BRACE) and (member_declaration_list := self.member_declaration_list()):
            return self.error('前面缺少一个右大括号', self.curtoken().location)
        self.restore(_z)
        if (struct_or_union := self.struct_or_union()) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and ((identifier := self.identifier()),) and self.expect(TokenKind.L_BRACE) and (a := self.expect(TokenKind.R_BRACE)):
            return self.error('记录声明不能没有成员', a.location)
        self.restore(_z)
        if (struct_or_union := self.struct_or_union()) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and ((identifier := self.identifier()),) and (a := self.expect(TokenKind.L_BRACE)):
            return self.error('后面缺少成员声明列表', a.location)
        self.restore(_z)
        if (struct_or_union := self.struct_or_union()) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),):
            return self.error('匿名的记录声明必须有成员声明列表', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def struct_or_union(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.STRUCT)):
            return a
        self.restore(_z)
        if (a := self.expect(TokenKind.UNION)):
            return a
        self.restore(_z)
        return None

    @memorize_left_rec
    def member_declaration_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.member_declaration_list()) and (b := self.member_declaration()):
            return a + [b]
        self.restore(_z)
        if (b := self.member_declaration()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def member_declaration(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.attribute_specifier_sequence()),) and (b := self.specifier_qualifier_list()) and ((c := self.member_declarator_list()),) and (d := self.expect(TokenKind.SEMI)):
            return FieldDecl(attribute_specifiers=option(a, []), specifiers=b[0], specifier_attributes=b[1], declarators=option(c, []), location=d.location)
        self.restore(_z)
        if (static_assert_declaration := self.static_assert_declaration()):
            return static_assert_declaration
        self.restore(_z)
        if ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and (specifier_qualifier_list := self.specifier_qualifier_list()) and ((member_declarator_list := self.member_declarator_list()),):
            return self.error("前面缺少一个';'", self.curtoken().location)
        self.restore(_z)
        if (attribute_specifier_sequence := self.attribute_specifier_sequence()):
            return self.error('前面缺少类型说明符或限定符', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def specifier_qualifier_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.type_specifier_qualifier()) and (b := self.specifier_qualifier_list()):
            return ([a] + b[0], b[1])
        self.restore(_z)
        if (a := self.type_specifier_qualifier()) and ((b := self.attribute_specifier_sequence()),):
            return ([a], option(b, []))
        self.restore(_z)
        return None

    @memorize
    def type_specifier_qualifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (type_specifier := self.type_specifier()):
            return type_specifier
        self.restore(_z)
        if (type_qualifier := self.type_qualifier()):
            return type_qualifier
        self.restore(_z)
        if (alignment_specifier := self.alignment_specifier()):
            return alignment_specifier
        self.restore(_z)
        return None

    @memorize_left_rec
    def member_declarator_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.member_declarator_list()) and self.expect(TokenKind.COMMA) and (b := self.member_declarator()):
            return a + [b]
        self.restore(_z)
        if (b := self.member_declarator()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def member_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.declarator()),) and (c := self.expect(TokenKind.COLON)) and (b := self.constant_expression()):
            return MemberDecl(declarator=a, bit_field=b, location=c.location)
        self.restore(_z)
        if (a := self.declarator()):
            return MemberDecl(declarator=a, bit_field=None, location=a.location)
        self.restore(_z)
        if ((declarator := self.declarator()),) and (a := self.expect(TokenKind.COLON)):
            return self.error('后面缺少位域的表达式', a.location)
        self.restore(_z)
        return None

    @memorize
    def enum_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.ENUM)) and ((b := self.attribute_specifier_sequence()),) and ((c := self.identifier()),) and ((d := self.enum_type_specifier()),) and self.expect(TokenKind.L_BRACE) and (e := self.enumerator_list()) and (self.expect(TokenKind.COMMA),) and self.expect(TokenKind.R_BRACE):
            return EnumDecl(attribute_specifiers=b, name=c.text if c != None else '', specifiers=d, enumerators=e, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.ENUM)) and (c := self.identifier()) and ((d := self.enum_type_specifier()),):
            return EnumDecl(attribute_specifiers=[], name=c.text, specifiers=d, enumerators=[], location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.ENUM) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and ((identifier := self.identifier()),) and ((enum_type_specifier := self.enum_type_specifier()),) and self.expect(TokenKind.L_BRACE) and (enumerator_list := self.enumerator_list()) and (self.expect(TokenKind.COMMA),):
            return self.error('前面缺少一个右大括号', self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.ENUM) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and ((identifier := self.identifier()),) and ((enum_type_specifier := self.enum_type_specifier()),) and self.expect(TokenKind.L_BRACE):
            return self.error('后面缺少成员声明列表', a.location)
        self.restore(_z)
        if self.expect(TokenKind.ENUM) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and ((identifier := self.identifier()),) and ((enum_type_specifier := self.enum_type_specifier()),) and self.expect(TokenKind.L_BRACE) and (a := self.expect(TokenKind.R_BRACE)):
            return self.error('枚举不能没有枚举项', a.location)
        self.restore(_z)
        if self.expect(TokenKind.ENUM) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and ((enum_type_specifier := self.enum_type_specifier()),):
            return self.error('匿名的枚举声明必须有枚举项', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def enumerator_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.enumerator_list()) and self.expect(TokenKind.COMMA) and (b := self.enumerator()):
            return a + [b]
        self.restore(_z)
        if (b := self.enumerator()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def enumeration_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (identifier := self.identifier()):
            return identifier
        self.restore(_z)
        return None

    @memorize
    def enumerator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.enumeration_constant()) and ((b := self.attribute_specifier_sequence()),) and self.expect(TokenKind.EQUAL) and (c := self.constant_expression()):
            return Enumerator(name=a.text, attribute_specifiers=option(b, []), value=c, location=a.location)
        self.restore(_z)
        if (a := self.enumeration_constant()) and ((b := self.attribute_specifier_sequence()),):
            return Enumerator(name=a.text, attribute_specifiers=option(b, []), value=None, location=a.location)
        self.restore(_z)
        if (enumeration_constant := self.enumeration_constant()) and ((attribute_specifier_sequence := self.attribute_specifier_sequence()),) and (a := self.expect(TokenKind.EQUAL)):
            return self.error('后面缺少一个表达式', a.location)
        self.restore(_z)
        return None

    @memorize
    def enum_type_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if self.expect(TokenKind.COLON) and (a := self.specifier_qualifier_list()):
            return a
        self.restore(_z)
        if (a := self.expect(TokenKind.COLON)):
            return self.error('后面缺少类型说明符或限定符', a.location)
        self.restore(_z)
        return None

    @memorize
    def atomic_type_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind._ATOMIC)) and self.expect(TokenKind.L_PAREN) and (b := self.type_name()) and self.expect(TokenKind.R_PAREN):
            return AtomicSpecifier(type_name=b, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind._ATOMIC) and self.expect(TokenKind.L_PAREN) and (a := self.type_name()):
            return self.error("后面缺少')'", a.location)
        self.restore(_z)
        return None

    @memorize
    def typeof_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.TYPEOF)) and self.expect(TokenKind.L_PAREN) and (b := self.typeof_specifier_argument()) and self.expect(TokenKind.R_PAREN):
            return TypeOfSpecifier(arg=b, is_unqual=False, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.TYPEOF_UNQUAL)) and self.expect(TokenKind.L_PAREN) and (b := self.typeof_specifier_argument()) and self.expect(TokenKind.R_PAREN):
            return TypeOfSpecifier(arg=b, is_unqual=True, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.TYPEOF) and self.expect(TokenKind.L_PAREN) and (typeof_specifier_argument := self.typeof_specifier_argument()):
            return self.error("前面缺少')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.TYPEOF) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个类型', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.TYPEOF)):
            return self.error("后面期待一个'('", a.location)
        self.restore(_z)
        if self.expect(TokenKind.TYPEOF_UNQUAL) and self.expect(TokenKind.L_PAREN) and (typeof_specifier_argument := self.typeof_specifier_argument()):
            return self.error("前面缺少')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.TYPEOF_UNQUAL) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个类型', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.TYPEOF_UNQUAL)):
            return self.error("后面期待一个'('", a.location)
        self.restore(_z)
        return None

    @memorize
    def typeof_specifier_argument(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (expression := self.expression()):
            return expression
        self.restore(_z)
        if (type_name := self.type_name()):
            return type_name
        self.restore(_z)
        return None

    @memorize
    def type_qualifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.CONST)):
            return TypeQualifier(qualifier=TypeQualifierKind.CONST, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.RESTRICT)):
            return TypeQualifier(qualifier=TypeQualifierKind.RESTRICT, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.VOLATILE)):
            return TypeQualifier(qualifier=TypeQualifierKind.VOLATILE, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind._ATOMIC)):
            return TypeQualifier(qualifier=TypeQualifierKind._ATOMIC, location=a.location)
        self.restore(_z)
        return None

    @memorize
    def function_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.INLINE)):
            return FunctionSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind._NORETURN)):
            return FunctionSpecifier(specifier_name=a.text, location=a.location)
        self.restore(_z)
        return None

    @memorize
    def alignment_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.ALIGNAS)) and self.expect(TokenKind.L_PAREN) and (b := self.type_name()) and self.expect(TokenKind.R_PAREN):
            return AlignSpecifier(type_or_expr=b, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.ALIGNAS)) and self.expect(TokenKind.L_PAREN) and (b := self.constant_expression()) and self.expect(TokenKind.R_PAREN):
            return AlignSpecifier(type_or_expr=b, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.ALIGNAS) and self.expect(TokenKind.L_PAREN) and (type_name := self.type_name()):
            return self.error("前面缺少')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.ALIGNAS) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个类型', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.ALIGNAS)):
            return self.error("后面期待一个'('", a.location)
        self.restore(_z)
        if self.expect(TokenKind.ALIGNAS) and self.expect(TokenKind.L_PAREN) and (constant_expression := self.constant_expression()):
            return self.error("前面缺少')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.ALIGNAS) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个类型', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.ALIGNAS)):
            return self.error("后面期待一个'('", a.location)
        self.restore(_z)
        return None

    @memorize
    def declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.pointer()),) and (b := self.direct_declarator()):
            return concat_pointer(a, b)
        self.restore(_z)
        if (a := self.pointer()):
            return self.error('后面缺少声明符', a.location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def direct_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.array_declarator()) and ((b := self.attribute_specifier_sequence()),):
            return assign(a, 'attribute_specifiers', option(b, []))
        self.restore(_z)
        if (a := self.function_declarator()) and ((b := self.attribute_specifier_sequence()),):
            return assign(a, 'attribute_specifiers', option(b, []))
        self.restore(_z)
        if (a := self.identifier()) and ((b := self.attribute_specifier_sequence()),):
            return NameDeclarator(name=a.text, declarator=None, attribute_specifiers=option(b, []), location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and (a := self.declarator()) and self.expect(TokenKind.R_PAREN):
            return a
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and (a := self.declarator()):
            return self.error("后面缺少')'", a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面缺少声明符', a.location)
        self.restore(_z)
        return None

    def array_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.direct_declarator()) and (d := self.expect(TokenKind.L_SQUARE)) and (b := self.type_qualifier_list()) and self.expect(TokenKind.STATIC) and (c := self.assignment_expression()) and self.expect(TokenKind.R_SQUARE):
            return ArrayDeclarator(declarator=a, qualifiers=b, size=c, is_star_modified=False, is_static=True, location=d.location)
        self.restore(_z)
        if (a := self.direct_declarator()) and (d := self.expect(TokenKind.L_SQUARE)) and self.expect(TokenKind.STATIC) and ((b := self.type_qualifier_list()),) and (c := self.assignment_expression()) and self.expect(TokenKind.R_SQUARE):
            return ArrayDeclarator(declarator=a, qualifiers=option(b, []), size=c, is_star_modified=False, is_static=True, location=d.location)
        self.restore(_z)
        if (a := self.direct_declarator()) and (d := self.expect(TokenKind.L_SQUARE)) and ((b := self.type_qualifier_list()),) and self.expect(TokenKind.STAR) and self.expect(TokenKind.R_SQUARE):
            return ArrayDeclarator(declarator=a, qualifiers=option(b, []), size=None, is_star_modified=True, is_static=False, location=d.location)
        self.restore(_z)
        if (a := self.direct_declarator()) and (d := self.expect(TokenKind.L_SQUARE)) and ((b := self.type_qualifier_list()),) and ((c := self.assignment_expression()),) and self.expect(TokenKind.R_SQUARE):
            return ArrayDeclarator(declarator=a, qualifiers=option(b, []), size=c, is_star_modified=False, is_static=False, location=d.location)
        self.restore(_z)
        if (direct_declarator := self.direct_declarator()) and self.expect(TokenKind.L_SQUARE) and (type_qualifier_list := self.type_qualifier_list()) and self.expect(TokenKind.STATIC) and (c := self.assignment_expression()):
            return self.error("后面缺少']'", c.location)
        self.restore(_z)
        if (direct_declarator := self.direct_declarator()) and self.expect(TokenKind.L_SQUARE) and self.expect(TokenKind.STATIC) and ((type_qualifier_list := self.type_qualifier_list()),) and (c := self.assignment_expression()):
            return self.error("后面缺少']'", c.location)
        self.restore(_z)
        if (direct_declarator := self.direct_declarator()) and self.expect(TokenKind.L_SQUARE) and ((type_qualifier_list := self.type_qualifier_list()),) and (c := self.expect(TokenKind.STAR)):
            return self.error("后面缺少']'", c.location)
        self.restore(_z)
        if (direct_declarator := self.direct_declarator()) and self.expect(TokenKind.L_SQUARE) and ((type_qualifier_list := self.type_qualifier_list()),) and ((assignment_expression := self.assignment_expression()),):
            return self.error("前面缺少']'", self.curtoken().location)
        self.restore(_z)
        if (direct_declarator := self.direct_declarator()) and self.expect(TokenKind.L_SQUARE) and (type_qualifier_list := self.type_qualifier_list()) and (a := self.expect(TokenKind.STATIC)):
            return self.error('后面缺少一个表达式', a.location)
        self.restore(_z)
        if (direct_declarator := self.direct_declarator()) and self.expect(TokenKind.L_SQUARE) and self.expect(TokenKind.STATIC) and ((type_qualifier_list := self.type_qualifier_list()),):
            return self.error('前面期待一个表达式', self.curtoken().location)
        self.restore(_z)
        return None

    def function_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.direct_declarator()) and (c := self.expect(TokenKind.L_PAREN)) and ((b := self.parameter_type_list()),) and self.expect(TokenKind.R_PAREN):
            return FunctionDeclarator(declarator=a, parameters=option(b, [[], False])[0], has_varparam=option(b, [[], False])[1], location=c.location)
        self.restore(_z)
        if (direct_declarator := self.direct_declarator()) and self.expect(TokenKind.L_PAREN) and ((parameter_type_list := self.parameter_type_list()),):
            return self.error("前面缺少')'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def pointer(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.STAR)) and ((c := self.attribute_specifier_sequence()),) and ((d := self.type_qualifier_list()),) and ((e := self.pointer()),):
            return PointerDeclarator(declarator=e, attribute_specifiers=option(c, []), qualifiers=option(d, []), location=a.location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def type_qualifier_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.type_qualifier_list()) and (b := self.type_qualifier()):
            return a + [b]
        self.restore(_z)
        if (b := self.type_qualifier()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def parameter_type_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.parameter_list()) and self.expect(TokenKind.COMMA) and self.expect(TokenKind.ELLIPSIS):
            return (a, True)
        self.restore(_z)
        if (a := self.parameter_list()):
            return (a, False)
        self.restore(_z)
        if self.expect(TokenKind.ELLIPSIS):
            return ([], True)
        self.restore(_z)
        if (parameter_list := self.parameter_list()) and (a := self.expect(TokenKind.COMMA)):
            return self.error("后面期待一个'...'", a.location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def parameter_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.parameter_list()) and self.expect(TokenKind.COMMA) and (b := self.parameter_declaration()):
            return a + [b]
        self.restore(_z)
        if (b := self.parameter_declaration()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def parameter_declaration(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.attribute_specifier_sequence()),) and (b := self.declaration_specifiers()) and (c := self.declarator()):
            return ParamDecl(attribute_specifiers=option(a, []), specifiers=b[0], specifier_attributes=b[1], declarator=c, location=begin_location)
        self.restore(_z)
        if ((a := self.attribute_specifier_sequence()),) and (b := self.declaration_specifiers()) and ((c := self.abstract_declarator()),):
            return ParamDecl(attribute_specifiers=option(a, []), specifiers=b[0], specifier_attributes=b[1], declarator=c, location=begin_location)
        self.restore(_z)
        return None

    @memorize
    def type_name(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.specifier_qualifier_list()) and ((b := self.abstract_declarator()),):
            return TypeName(specifiers=a[0], specifier_attributes=a[1], declarator=b, location=begin_location)
        self.restore(_z)
        return None

    @memorize
    def abstract_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.pointer()),) and (b := self.direct_abstract_declarator()):
            return concat_pointer(a, b)
        self.restore(_z)
        if (pointer := self.pointer()):
            return pointer
        self.restore(_z)
        return None

    @memorize_left_rec
    def direct_abstract_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.array_abstract_declarator()) and ((b := self.attribute_specifier_sequence()),):
            return assign(a, 'attribute_specifiers', option(b, []))
        self.restore(_z)
        if (a := self.function_abstract_declarator()) and ((b := self.attribute_specifier_sequence()),):
            return assign(a, 'attribute_specifiers', option(b, []))
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and (a := self.abstract_declarator()) and self.expect(TokenKind.R_PAREN):
            return a
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and (a := self.abstract_declarator()):
            return self.error("后面缺少')'", a.location)
        self.restore(_z)
        return None

    def array_abstract_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.direct_abstract_declarator()),) and (d := self.expect(TokenKind.L_SQUARE)) and self.expect(TokenKind.STATIC) and ((b := self.type_qualifier_list()),) and (c := self.assignment_expression()) and self.expect(TokenKind.R_SQUARE):
            return ArrayDeclarator(declarator=a, qualifiers=option(b, []), size=c, is_star_modified=False, is_static=True, location=d.location)
        self.restore(_z)
        if ((a := self.direct_abstract_declarator()),) and (d := self.expect(TokenKind.L_SQUARE)) and self.expect(TokenKind.STAR) and self.expect(TokenKind.R_SQUARE):
            return ArrayDeclarator(declarator=a, qualifiers=[], size=None, is_star_modified=True, is_static=False, location=d.location)
        self.restore(_z)
        if ((a := self.direct_abstract_declarator()),) and (d := self.expect(TokenKind.L_SQUARE)) and (b := self.type_qualifier_list()) and self.expect(TokenKind.STATIC) and (c := self.assignment_expression()) and self.expect(TokenKind.R_SQUARE):
            return ArrayDeclarator(declarator=a, qualifiers=b, size=c, is_star_modified=False, is_static=True, location=d.location)
        self.restore(_z)
        if ((a := self.direct_abstract_declarator()),) and (d := self.expect(TokenKind.L_SQUARE)) and ((b := self.type_qualifier_list()),) and ((c := self.assignment_expression()),) and self.expect(TokenKind.R_SQUARE):
            return ArrayDeclarator(declarator=a, qualifiers=option(b, []), size=c, is_star_modified=False, is_static=False, location=d.location)
        self.restore(_z)
        if ((direct_abstract_declarator := self.direct_abstract_declarator()),) and self.expect(TokenKind.L_SQUARE) and self.expect(TokenKind.STATIC) and ((type_qualifier_list := self.type_qualifier_list()),) and (c := self.assignment_expression()):
            return self.error("后面缺少']'", c.location)
        self.restore(_z)
        if ((direct_abstract_declarator := self.direct_abstract_declarator()),) and self.expect(TokenKind.L_SQUARE) and (c := self.expect(TokenKind.STAR)):
            return self.error("后面缺少']'", c.location)
        self.restore(_z)
        if ((direct_abstract_declarator := self.direct_abstract_declarator()),) and self.expect(TokenKind.L_SQUARE) and (type_qualifier_list := self.type_qualifier_list()) and self.expect(TokenKind.STATIC) and (c := self.assignment_expression()):
            return self.error("后面缺少']'", c.location)
        self.restore(_z)
        if ((direct_abstract_declarator := self.direct_abstract_declarator()),) and self.expect(TokenKind.L_SQUARE) and ((type_qualifier_list := self.type_qualifier_list()),) and ((assignment_expression := self.assignment_expression()),):
            return self.error("前面缺少']'", self.curtoken().location)
        self.restore(_z)
        if ((direct_abstract_declarator := self.direct_abstract_declarator()),) and self.expect(TokenKind.L_SQUARE) and (type_qualifier_list := self.type_qualifier_list()) and (a := self.expect(TokenKind.STATIC)):
            return self.error('后面缺少一个表达式', a.location)
        self.restore(_z)
        if ((direct_abstract_declarator := self.direct_abstract_declarator()),) and self.expect(TokenKind.L_SQUARE) and self.expect(TokenKind.STATIC) and ((type_qualifier_list := self.type_qualifier_list()),):
            return self.error('前面期待一个表达式', self.curtoken().location)
        self.restore(_z)
        return None

    def function_abstract_declarator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.direct_abstract_declarator()),) and (d := self.expect(TokenKind.L_PAREN)) and ((b := self.parameter_type_list()),) and self.expect(TokenKind.R_PAREN):
            return FunctionDeclarator(declarator=a, parameters=option(b, [[], False])[0], has_varparam=option(b, [[], False])[1], location=d.location)
        self.restore(_z)
        if ((direct_abstract_declarator := self.direct_abstract_declarator()),) and self.expect(TokenKind.L_PAREN) and ((b := self.parameter_type_list()),):
            return self.error("前面缺少')'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def braced_initializer(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.L_BRACE)) and (b := self.initializer_list()) and (self.expect(TokenKind.COMMA),) and self.expect(TokenKind.R_BRACE):
            return InitList(initializers=b, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.L_BRACE)) and self.expect(TokenKind.R_BRACE):
            return InitList(initializers=[], location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.L_BRACE) and (initializer_list := self.initializer_list()) and (self.expect(TokenKind.COMMA),):
            return self.error('前面缺少一个右大括号', self.curtoken().location)
        self.restore(_z)
        if (a := self.expect(TokenKind.L_BRACE)):
            return self.error('后面期待一个右大括号或一个初始化列表', a.location)
        self.restore(_z)
        return None

    @memorize
    def initializer(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (assignment_expression := self.assignment_expression()):
            return assignment_expression
        self.restore(_z)
        if (braced_initializer := self.braced_initializer()):
            return braced_initializer
        self.restore(_z)
        return None

    @memorize_left_rec
    def initializer_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.initializer_list()) and self.expect(TokenKind.COMMA) and ((b := self.designation()),) and (c := self.initializer()):
            return a + [assign(b, 'initializer', c) if b != None else c]
        self.restore(_z)
        if ((b := self.designation()),) and (c := self.initializer()):
            return [assign(b, 'initializer', c) if b != None else c]
        self.restore(_z)
        return None

    @memorize
    def designation(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.designator_list()) and (b := self.expect(TokenKind.EQUAL)):
            return Designation(designators=a, location=b.location)
        self.restore(_z)
        if (designator_list := self.designator_list()):
            return self.error("前面缺少'='", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def designator_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.designator_list()) and (b := self.designator()):
            return a + [b]
        self.restore(_z)
        if (b := self.designator()):
            return [b]
        self.restore(_z)
        return None

    @memorize
    def designator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.L_SQUARE)) and (b := self.constant_expression()) and self.expect(TokenKind.R_SQUARE):
            return Designator(index=b, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.PERIOD)) and (b := self.identifier()):
            return Designator(member=b.text, location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.L_SQUARE) and (b := self.constant_expression()):
            return self.error("后面缺少']'", b.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.PERIOD)):
            return self.error('后面缺少标识符', a.location)
        self.restore(_z)
        return None

    @memorize
    def static_assert_declaration(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.STATIC_ASSERT)) and self.expect(TokenKind.L_PAREN) and (b := self.constant_expression()) and self.expect(TokenKind.COMMA) and (c := self.string_literal()) and self.expect(TokenKind.R_PAREN) and self.expect(TokenKind.SEMI):
            return StaticAssert(condition_expr=b, message=c.text, location=a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.STATIC_ASSERT)) and self.expect(TokenKind.L_PAREN) and (b := self.constant_expression()) and self.expect(TokenKind.R_PAREN) and self.expect(TokenKind.SEMI):
            return StaticAssert(condition_expr=b, message='', location=a.location)
        self.restore(_z)
        if self.expect(TokenKind.STATIC_ASSERT) and self.expect(TokenKind.L_PAREN) and (constant_expression := self.constant_expression()) and self.expect(TokenKind.COMMA) and (string_literal := self.string_literal()) and (d := self.expect(TokenKind.R_PAREN)):
            return self.error("后面缺少';'", d.location)
        self.restore(_z)
        if self.expect(TokenKind.STATIC_ASSERT) and self.expect(TokenKind.L_PAREN) and (constant_expression := self.constant_expression()) and (d := self.expect(TokenKind.R_PAREN)):
            return self.error("后面缺少';'", d.location)
        self.restore(_z)
        if self.expect(TokenKind.STATIC_ASSERT) and self.expect(TokenKind.L_PAREN) and (constant_expression := self.constant_expression()) and self.expect(TokenKind.COMMA) and (c := self.string_literal()):
            return self.error("后面缺少')'", c.location)
        self.restore(_z)
        if self.expect(TokenKind.STATIC_ASSERT) and self.expect(TokenKind.L_PAREN) and (constant_expression := self.constant_expression()) and (c := self.expect(TokenKind.COMMA)):
            return self.error('后面缺少字符串', c.location)
        self.restore(_z)
        if self.expect(TokenKind.STATIC_ASSERT) and self.expect(TokenKind.L_PAREN) and (constant_expression := self.constant_expression()):
            return self.error("前面缺少')'", self.curtoken().location)
        self.restore(_z)
        if self.expect(TokenKind.STATIC_ASSERT) and (a := self.expect(TokenKind.L_PAREN)):
            return self.error('后面期待一个类型', a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.STATIC_ASSERT)):
            return self.error("后面期待一个'('", a.location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def attribute_specifier_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.attribute_specifier_sequence()),) and (b := self.attribute_specifier()):
            return option(a, []) + [b]
        self.restore(_z)
        return None

    @memorize
    def attribute_specifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (b := self.expect(TokenKind.L_SQUARE)) and self.expect(TokenKind.L_SQUARE) and (a := self.attribute_list()) and self.expect(TokenKind.R_SQUARE) and self.expect(TokenKind.R_SQUARE):
            return AttributeSpecifier(attributes=a, location=b.location)
        self.restore(_z)
        if self.expect(TokenKind.L_SQUARE) and self.expect(TokenKind.L_SQUARE) and (attribute_list := self.attribute_list()) and (a := self.expect(TokenKind.R_SQUARE)):
            return self.error("后面还少一个']'", a.location)
        self.restore(_z)
        if self.expect(TokenKind.L_SQUARE) and self.expect(TokenKind.L_SQUARE) and (attribute_list := self.attribute_list()):
            return self.error("前面缺少']'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def attribute_list(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.attribute_list()) and self.expect(TokenKind.COMMA) and ((b := self.attribute()),):
            return a + ([b] if b != None else [])
        self.restore(_z)
        if ((b := self.attribute()),):
            return [b] if b != None else []
        self.restore(_z)
        return None

    @memorize
    def attribute(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.attribute_token()) and ((b := self.attribute_argument_clause()),):
            return assign(a, 'args', b)
        self.restore(_z)
        return None

    @memorize
    def attribute_token(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.attribute_prefixed_token()):
            return Attribute(prefix_name=a[0].text, name=a[1].text, location=begin_location)
        self.restore(_z)
        if (a := self.standard_attribute()):
            return Attribute(prefix_name='', name=a.text, location=a.location)
        self.restore(_z)
        return None

    @memorize
    def attribute_prefixed_token(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.attribute_prefix()) and self.expect(TokenKind.COLONCOLON) and (b := self.identifier()):
            return (a, b)
        self.restore(_z)
        if (attribute_prefix := self.attribute_prefix()) and (a := self.expect(TokenKind.COLONCOLON)):
            return self.error('后面缺少一个标识符', a.location)
        self.restore(_z)
        return None

    @memorize
    def attribute_prefix(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (identifier := self.identifier()):
            return identifier
        self.restore(_z)
        return None

    @memorize
    def standard_attribute(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (identifier := self.identifier()):
            return identifier
        self.restore(_z)
        return None

    @memorize
    def attribute_argument_clause(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if self.expect(TokenKind.L_PAREN) and ((a := self.balanced_token_sequence()),) and self.expect(TokenKind.R_PAREN):
            return option(a, [])
        self.restore(_z)
        if self.expect(TokenKind.L_PAREN) and ((balanced_token_sequence := self.balanced_token_sequence()),):
            return self.error("前面缺少')'", self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def balanced_token_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.balanced_token_sequence()) and (b := self.balanced_token()):
            return a + b
        self.restore(_z)
        if (b := self.balanced_token()):
            return b
        self.restore(_z)
        return None