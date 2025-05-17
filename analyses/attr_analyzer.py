from analyses.analyzer import Analyzer
from cast import AttributeSpecifier, Attribute, DeprecatedAttr, NodiscardAttr
from basic import Warn, ATTRIBUTE_NAMES, Error, TokenKind


class AttrAnalyzer(Analyzer):
    """
    完成的任务:
    1. 获取Attribute对应的真正属性
    2. 检查部分内置属性的参数
    """

    def visit_Attribute(self, node: Attribute):
        namespace: dict = self.cur_symtab.lookup(node.prefix_name, ATTRIBUTE_NAMES)
        if namespace == None:
            attrbute_cls = None
        else:
            attrbute_cls = namespace.get(node.name, None)
        if attrbute_cls == None:
            Warn(f"未知的属性: {node.prefix_name}::{node.name}", node.location).dump()
        else:
            attribute: Attribute = attrbute_cls(**node.__dict__)
            return attribute.accept(self)
        return node

    def visit_AttributeSpecifier(self, node: AttributeSpecifier):
        for i, v in enumerate(node.attributes):
            node.attributes[i] = v.accept(self)

    def visit_DeprecatedAttr(self, node: DeprecatedAttr):
        if not hasattr(node, "args") or node.args == None:
            return node
        if len(node.args) != 1:
            raise Error("deprecated 只能有一个参数或没有参数", node.location)
        if node.args[0].kind != TokenKind.STRINGLITERAL:
            raise Error("deprecated 的参数必须是字符串字面量", node.args[0].location)
        return node

    def visit_NodiscardAttr(self, node: NodiscardAttr):
        if hasattr(node, "args") and node.args != None and len(node.args) != 1:
            raise Error("nodiscard 只能有一个参数或没有参数", node.location)
        if node.args[0].kind != TokenKind.STRINGLITERAL:
            raise Error("nodiscard 的参数必须是字符串字面量", node.args[0].location)
        return node
