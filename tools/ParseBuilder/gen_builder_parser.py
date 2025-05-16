from parse import ParserBase, memorize, memorize_left_rec
from basic import *
from basic import *

def unpack_header_rule(a):
    header = ''
    rule = []
    for i in a:
        rule.append(i[0])
        if i[1] != None:
            header += i[1].content
    return {'header': header, 'rules': rule}

class Gen_BuilderParser(ParserBase):

    @memorize
    def start(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.grammar()) and (end := self.end()):
            return a
        self.restore(_z)
        if (grammar := self.grammar()):
            return self.error('解析已结束, 但文件未结束', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def grammar(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (rules := self.rules()):
            return Grammar(**unpack_header_rule(rules), location=begin_location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def rules(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (rules := self.rules()) and ((header := self.header()),) and (rule := self.rule()):
            return rules + [(rule, header)]
        self.restore(_z)
        if ((header := self.header()),) and (rule := self.rule()):
            return [(rule, header)]
        self.restore(_z)
        return None

    @memorize
    def rule(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.identifier()) and self.expect(TokenKind.COLON) and ((b := self.rhs()),):
            return Rule(name=a.text, rhs=b, location=begin_location)
        self.restore(_z)
        if (a := self.identifier()):
            return self.error("后面应该有个';'", a.location)
        self.restore(_z)
        if (a := self.expect(TokenKind.PIPE)):
            return self.error("'|'后缺少可选体", a.location)
        self.restore(_z)
        return None

    @memorize
    def rhs(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (alts := self.alts()):
            return Rhs(alts=alts, location=begin_location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def alts(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (alts := self.alts()) and self.expect(TokenKind.PIPE) and (alt := self.alt()):
            return alts + [alt]
        self.restore(_z)
        if (alt := self.alt()):
            return [alt]
        self.restore(_z)
        return None

    @memorize
    def alt(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (items := self.items()) and ((action := self.action()),):
            return Alt(items=items, location=begin_location, action=action.content if action != None else None)
        self.restore(_z)
        return None

    @memorize_left_rec
    def items(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (items := self.items()) and (item := self.item()):
            return items + [item]
        self.restore(_z)
        if (item := self.item()):
            return [item]
        self.restore(_z)
        return None

    @memorize
    def item(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (named_item := self.named_item()):
            return named_item
        self.restore(_z)
        return None

    @memorize
    def named_item(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.identifier()) and self.expect(TokenKind.EQUAL) and (b := self.opt_item()):
            return NamedItem(name=a.text, item=b, location=a.location)
        self.restore(_z)
        if (opt_item := self.opt_item()):
            return opt_item
        self.restore(_z)
        if (identifier := self.identifier()) and (c := self.expect(TokenKind.EQUAL)):
            return self.error('缺少要命名的项', c.location)
        self.restore(_z)
        return None

    @memorize
    def opt_item(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.L_SQUARE)) and (b := self.leaf_item()) and self.expect(TokenKind.R_SQUARE):
            return Option(item=b, location=a.location)
        self.restore(_z)
        if (leaf_item := self.leaf_item()):
            return leaf_item
        self.restore(_z)
        if self.expect(TokenKind.L_SQUARE) and (leaf_item := self.leaf_item()) and (c := self.expect(TokenKind.R_SQUARE)):
            return self.error("'['未闭合", location=c.location)
        self.restore(_z)
        return None

    @memorize
    def leaf_item(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.identifier()):
            return NameLeaf(name=a.text, location=a.location)
        self.restore(_z)
        if (a := self.string_literal()):
            return StringLeaf(value=a.content, location=a.location)
        self.restore(_z)
        return None