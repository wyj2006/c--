import ast
from AST import (
    Grammar,
    Visitor,
    NameLeaf,
    StringLeaf,
    Rhs,
    Rule,
    Alt,
    NamedItem,
    Option,
)
from Basic import Token

# TODO: 空列表被判定为匹配失败


class Generator(Visitor):
    def visit_Grammar(self, node: Grammar):
        classdef = ast.ClassDef(
            name="Gen_" + node.class_name,
            bases=[ast.Name(id="ParserBase", ctx=ast.Load())],
            keywords=[],
            body=[],
            decorator_list=[],
        )
        module = ast.Module(
            body=[
                ast.ImportFrom(
                    module="Parse.Builder",
                    names=[
                        ast.alias(name="ParserBase"),
                        ast.alias(name="memorize"),
                        ast.alias(name="memorize_left_rec"),
                    ],
                    level=0,
                ),
            ],
            type_ignores=[],
        )

        if hasattr(node, "header") and node.header != None:
            module.body.extend(ast.parse(node.header).body)
        module.body.append(classdef)

        for rule in node.rules:
            classdef.body.append(rule.accept(self))
        return module

    def visit_Rule(self, node: Rule):
        if not hasattr(node, "is_left_rec"):
            node.is_left_rec = False
        funcdef = ast.FunctionDef(
            name=node.name,
            args=ast.arguments(
                posonlyargs=[],
                args=[ast.arg(arg="self")],
                kwonlyargs=[],
                kw_defaults=[],
                defaults=[],
            ),
            body=[],
            decorator_list=[
                ast.Name(
                    id="memorize" if not node.is_left_rec else "memorize_left_rec",
                    ctx=ast.Load(),
                )
            ],
        )
        if node.rhs != None:
            funcdef.body.extend(node.rhs.accept(self))
        return ast.fix_missing_locations(funcdef)

    def visit_Rhs(self, node: Rhs):
        restore_var_name = "_z"
        body = [
            ast.Assign(
                targets=[ast.Name(id="begin_location", ctx=ast.Store())],
                value=ast.Attribute(
                    value=ast.Call(
                        func=ast.Attribute(
                            value=ast.Name(id="self", ctx=ast.Load()),
                            attr="curtoken",
                            ctx=ast.Load(),
                        ),
                        args=[],
                        keywords=[],
                    ),
                    attr="location",
                    ctx=ast.Load(),
                ),
            ),
            ast.Assign(
                targets=[ast.Name(id=restore_var_name, ctx=ast.Store())],
                value=ast.Call(
                    func=ast.Attribute(
                        value=ast.Name(id="self", ctx=ast.Load()),
                        attr="save",
                        ctx=ast.Load(),
                    ),
                    args=[],
                    keywords=[],
                ),
            ),
        ]
        for alt in node.alts:
            body.extend(alt.accept(self))
            body.append(
                ast.Expr(
                    value=ast.Call(
                        func=ast.Attribute(
                            value=ast.Name(id="self", ctx=ast.Load()),
                            attr="restore",
                            ctx=ast.Load(),
                        ),
                        args=[ast.Name(id=restore_var_name, ctx=ast.Load())],
                        keywords=[],
                    )
                )
            )
        body.append(ast.Return(value=ast.Constant(value=None)))
        return body

    def visit_Alt(self, node: Alt):
        test_expr = ast.BoolOp(
            op=ast.And(),
            values=[],
        )

        if not hasattr(node, "action") or node.action == None:
            names = []
            for i in node.items:
                if hasattr(i, "name"):
                    names.append(i.name)
                elif isinstance(i, Option) and hasattr(i.item, "name"):
                    names.append(i.item.name)
            node.action = ",".join(names)
        node.action = node.action.replace("BEGIN_LOCATION", "begin_location").replace(
            "ERROR", "self.error"
        )

        ifstmt = ast.If(
            test=test_expr,
            body=[ast.Return(value=ast.parse(node.action, mode="eval").body)],
            orelse=[],
        )
        for item in node.items:
            test_expr.values.append(item.accept(self))
        return [ifstmt]

    def visit_NamedItem(self, node: NamedItem):
        item_code = node.item.accept(self)
        if isinstance(item_code, ast.NamedExpr):
            item_code.target.id = node.name
        elif isinstance(item_code, ast.Tuple):
            if isinstance(item_code.elts[0], ast.NamedExpr):
                item_code.elts[0].target.id = node.name
            else:
                item_code.elts[0] = ast.NamedExpr(
                    target=ast.Name(id=node.name, ctx=ast.Store()),
                    value=item_code.elts[0],
                )
        else:
            item_code = ast.NamedExpr(
                target=ast.Name(id=node.name, ctx=ast.Store()), value=item_code
            )
        return item_code

    def visit_Option(self, node: Option):
        return ast.Tuple(elts=[node.item.accept(self)])

    def visit_NameLeaf(self, node: NameLeaf):
        return ast.NamedExpr(
            target=ast.Name(id=node.name, ctx=ast.Store()),
            value=ast.Call(
                func=ast.Attribute(
                    value=ast.Name(id="self", ctx=ast.Load()),
                    attr=node.name,
                    ctx=ast.Load(),
                ),
                args=[],
                keywords=[],
            ),
        )

    def visit_StringLeaf(self, node: StringLeaf):
        token_kind = None
        if node.value in Token.punctuator:
            token_kind = Token.punctuator[node.value].name
        elif node.value in Token.keywords:
            token_kind = Token.keywords[node.value].name
        elif node.value in Token.ppkeywords:
            token_kind = Token.ppkeywords[node.value].name
        if token_kind != None:
            return ast.Call(
                func=ast.Attribute(
                    value=ast.Name(id="self", ctx=ast.Load()),
                    attr="expect",
                    ctx=ast.Load(),
                ),
                args=[
                    ast.Attribute(
                        value=ast.Name(id="TokenKind", ctx=ast.Load()),
                        attr=token_kind,
                        ctx=ast.Load(),
                    )
                ],
                keywords=[],
            )
        return ast.Call(
            func=ast.Attribute(
                value=ast.Name(id="self", ctx=ast.Load()), attr="expect", ctx=ast.Load()
            ),
            args=[
                ast.Attribute(
                    value=ast.Name(id="TokenKind", ctx=ast.Load()),
                    attr="IDENTIFIER",
                    ctx=ast.Load(),
                )
            ],
            keywords=[ast.keyword(arg="text", value=ast.Constant(value=node.value))],
        )
