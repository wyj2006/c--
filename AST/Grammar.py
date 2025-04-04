from AST.Node import Node


class Grammar(Node):
    """文法的语法"""

    class_name: str
    header: str
    rules: list["Rule"]

    rules_map: dict[str, "Rule"]

    _attributes = Node._attributes + ("class_name",)
    _fields = Node._fields + ("rules",)

    def merge(self):
        """合并重复的rule"""
        self.rules_map = {}
        i = 0
        while i < len(self.rules):
            rule = self.rules[i]
            if rule.name not in self.rules_map:
                self.rules_map[rule.name] = rule
            else:
                self.rules_map[rule.name].rhs.alts.extend(rule.rhs.alts)
                self.rules.pop(i)
                continue
            i += 1


class Rule(Node):
    name: str
    rhs: "Rhs"
    is_left_rec: bool  # 是否是左递归, 包括直接和间接以及参与左递归

    _attributes = Node._attributes + ("name", "is_left_rec")
    _fields = Node._fields + ("rhs",)


class Rhs(Node):
    alts: list["Alt"]

    _fields = Node._fields + ("alts",)


class Alt(Node):
    items: list["Item"]
    action: str

    _attributes = Node._attributes + ("action",)
    _fields = Node._fields + ("items",)


class Item(Node):
    pass


class NamedItem(Item):
    """被命名的项"""

    name: str
    item: Item

    _attributes = Item._attributes + ("name",)
    _fields = Item._fields + ("item",)


class Option(Item):
    item: Item

    _fields = Item._fields + ("item",)


class NameLeaf(Item):
    name: str

    _attributes = Item._attributes + ("name",)


class StringLeaf(Item):
    value: str
    _attributes = Item._attributes + ("value",)
