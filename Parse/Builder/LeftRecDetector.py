from AST import Visitor, Grammar, NamedItem, Option, NameLeaf, Alt, Rhs, Rule
from Basic import Error, Warn


class LeftRecDetector(Visitor):
    """检测左递归"""

    def visit_Grammar(self, node: Grammar):
        assert hasattr(node, "rules_map"), "需要提前调用merge方法"

        g: dict[str, list[Rule]] = {}  # 图

        for rule in node.rules:
            if rule.name == "start":
                start_rule = rule
            rule.is_left_rec = False

            rhs = rule.rhs
            non_terminal = set()
            for alt in rhs.alts:
                nameleaf: NameLeaf = alt.accept(self)
                if nameleaf == None:
                    continue
                if nameleaf.name not in node.rules_map:
                    Warn(
                        f"{nameleaf.name}未定义, 假定它不会产生左递归",
                        nameleaf.location,
                    ).dump()
                    continue
                non_terminal.add(node.rules_map[nameleaf.name])
            g[rule.name] = list(non_terminal)

        # 针对每个规则找环
        for start_rule_name in g:
            start_rule: Rule = node.rules_map[start_rule_name]
            if start_rule.is_left_rec:
                continue
            queue: list[tuple[Rule, tuple[Rule]]] = []
            # queue[i][0]表示对应的规则, queue[i][1]表示规则的路径
            queue.append((start_rule, (start_rule,)))
            while queue:
                rule, path = queue.pop(0)
                next_rule: Rule
                for next_rule in g[rule.name]:
                    if next_rule not in path:
                        queue.append((next_rule, path + (next_rule,)))
                        continue
                    # 产生左递归
                    for r in path[::-1]:
                        r.is_left_rec = True
                        if r.name == next_rule.name:
                            break

    def visit_Alt(self, node: Alt):
        return node.items[0].accept(self)

    # 每个节点都返回对应的非终结符(NameLeaf), 没有即为None
    def visit_NamedItem(self, node: NamedItem):
        return node.item.accept(self)

    def visit_Option(self, node: Option):
        return node.item.accept(self)

    def visit_NameLeaf(self, node: NameLeaf):
        return node
