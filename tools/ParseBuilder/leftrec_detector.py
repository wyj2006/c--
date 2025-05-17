from cast import (
    Visitor,
    Grammar,
    NamedItem,
    Option,
    NameLeaf,
    Alt,
    Rule,
    LeafItem,
)
from basic import Warn


class LeftRecDetector(Visitor):
    """检测左递归"""

    def visit_Grammar(self, node: Grammar):
        assert hasattr(node, "rules_map"), "需要提前调用merge方法"

        self.g: dict[str, list[Rule]] = {}  # 图
        self.rules_map = node.rules_map
        self.generic_visit(node)

        g = self.g

        for start_rule_name in self.rules_map:
            start_rule: Rule = self.rules_map[start_rule_name]
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
                        r.is_leader = False
                        if r.name == next_rule.name:
                            r.is_leader = True
                            break

    def visit_Rule(self, node: Rule):
        self.generic_visit(node)

        node.is_left_rec = False

        g = self.g

        non_terminal = set()
        for alt in node.rhs.alts:
            for leaf_item in alt.first_item:
                if not isinstance(leaf_item, NameLeaf):
                    continue
                if leaf_item.name not in self.rules_map:
                    Warn(
                        f"{leaf_item.name}未定义, 假定它不会产生左递归",
                        leaf_item.location,
                    ).dump()
                    continue
                non_terminal.add(self.rules_map[leaf_item.name])
        g[node.name] = list(non_terminal)

    def visit_Alt(self, node: Alt):
        self.generic_visit(node)

        node.first_item = set()

        for item in node.items:
            node.first_item |= item.first_item
            if not item.nullable:
                break

    # 每个节点都返回对应的非终结符(NameLeaf), 没有即为None
    def visit_NamedItem(self, node: NamedItem):
        self.generic_visit(node)

        node.nullable = node.item.nullable
        node.first_item = node.item.first_item

    def visit_Option(self, node: Option):
        self.generic_visit(node)

        node.nullable = True
        node.first_item = node.item.first_item

    def visit_LeafItem(self, node: LeafItem):
        node.nullable = False
        node.first_item = {node}
