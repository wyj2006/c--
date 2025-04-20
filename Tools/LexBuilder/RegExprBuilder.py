from copy import deepcopy
from AST import (
    Visitor,
    Grammar,
    RegExpr,
    Concat,
    Choice,
    Letter,
    Alt,
    NameLeaf,
    StringLeaf,
    Rule,
    Rhs,
    NamedItem,
    Option,
    EmptyString,
    Transformer,
    Item,
    Repeat0,
)
from Basic import Error


class BackPatchMark(RegExpr):
    """
    回填标记
    用于回填规则对应的正则表达式
    """

    rule_name: str

    _attributes = RegExpr._attributes + ("rule_name",)


class BackPatcher(Transformer):
    """回填"""

    def __init__(self, rules_regexpr: dict[str, RegExpr]):
        super().__init__()
        self.rules_regexpr = rules_regexpr

    def visit_BackPatchMark(self, node: BackPatchMark):
        return deepcopy(self.rules_regexpr[node.rule_name].accept(self))


class RegExprBuilder(Visitor):
    """
    根据文法构建相应的正则表达式
    注意不是所有的文法都能转换成正则表达式
    """

    rules_map: dict[str, Rule]
    rules_regexpr: dict[str, RegExpr]

    def visit_Grammar(self, node: Grammar):
        self.rules_map = node.rules_map
        self.rules_regexpr = {}

        a = Choice(exprs=[], location=node.location)
        for rule in node.rules:
            a.exprs.append(rule.accept(self))
            if rule.name == "start":
                start = a.exprs[-1]
        a.accept(BackPatcher(self.rules_regexpr))

        return start

    def visit_Rule(self, node: Rule):
        if not node.is_left_rec:
            self.rules_regexpr[node.name] = node.rhs.accept(self)
            return self.rules_regexpr[node.name]

        left_rec: RegExpr = Choice(exprs=[])
        non_left_rec: RegExpr = Choice(exprs=[])
        for alt in node.rhs.alts:
            if all(
                [
                    item.name != node.name
                    for item in alt.first_item
                    if isinstance(item, NameLeaf)
                ]
            ):
                # 没有产生左递归的部分
                if not hasattr(non_left_rec, "location"):
                    non_left_rec.location = alt.location
                non_left_rec.exprs.append(alt.accept(self))
            else:
                # 产生左递归的部分
                if not hasattr(left_rec, "location"):
                    left_rec.location = alt.location
                left_rec.exprs.append(
                    Alt(
                        items=alt.items[1:],
                        location=alt.items[1].location,
                    ).accept(self)
                )

        self.rules_regexpr[node.name] = Concat(
            exprs=[
                non_left_rec,
                Repeat0(repeat_expr=left_rec, location=left_rec.location),
            ],
            location=non_left_rec.location,
        )
        return self.rules_regexpr[node.name]

    def visit_Rhs(self, node: Rhs):
        a = Choice(exprs=[], location=node.location)
        for alt in node.alts:
            a.exprs.append(alt.accept(self))
        return a

    def visit_Alt(self, node: Alt):
        a = Concat(exprs=[], location=node.location)
        for i, item in enumerate(node.items):
            a.exprs.append(item.accept(self))
        if node.action != None:
            a.exprs.append(
                Letter(
                    char="pattern_end",
                    is_func=True,
                    location=a.exprs[-1].location,
                    action=node.action,
                )
            )
        return a

    def visit_NamedItem(self, node: NamedItem):
        return node.item.accept(self)

    def visit_Option(self, node: Option):
        return Choice(
            exprs=[node.item.accept(self), EmptyString(location=node.location)],
            location=node.location,
        )

    def visit_StringLeaf(self, node: StringLeaf):
        if len(node.value) > 1:
            raise Error("字符长度只能为1", node.location)
        return Letter(char=node.value, is_func=False, location=node.location)

    def visit_NameLeaf(self, node: NameLeaf):
        if node.name not in self.rules_map:
            return Letter(char=node.name, is_func=True, location=node.location)
        return BackPatchMark(rule_name=node.name, location=node.location)
