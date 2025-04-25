"""
词法分析器自动生成工具
"""

import ast
from AST import Letter, RegExpr
from .State import State


def generate_state(regexpr: RegExpr, followpos: dict[Letter, set[Letter]]):
    """生成状态"""
    id_count = 0
    start_state = State(id=id_count, involvedpos=regexpr.firstpos)
    id_count += 1
    states: list[State] = []

    queue = [start_state]
    while queue:
        state = queue.pop(0)
        states.append(state)

        transition = {}

        for pos in state.involvedpos:
            char = pos.char
            if pos.is_func:
                char += "()"
            if char not in transition:
                transition[char] = set()
            transition[char] |= followpos[pos]

        for char, involvedpos in transition.items():
            new_state = State(id=id_count, involvedpos=involvedpos)
            # 查找该状态是否出现过
            index = -1
            try:
                index = queue.index(new_state)
                new_state = queue[index]
            except ValueError:
                pass
            try:
                index = states.index(new_state)
                new_state = states[index]
            except ValueError:
                pass

            if index == -1:
                id_count += 1
                queue.append(new_state)
            state.transition[char] = new_state
    return states


def generate_code(states: list[State], class_name: str, header: str):
    """生成代码"""
    # 生成状态转移代码
    match_expr = ast.Match(
        subject=ast.parse("states[-1][0]", mode="eval").body,
        cases=[],
    )
    for state in states:
        pattern = ast.MatchValue(value=ast.Constant(value=state.id))
        body = []

        target_state_trans: dict[int, list[ast.If]] = {}  # 记录转到不同状态的语句

        # 不同的优先级
        normal = []  # 普通的字符比较
        func = []  # 需要调用函数
        for char, target_state in state.transition.items():
            if char == "pattern_end()":
                continue
            trans_stmt = ast.If(
                test=None,
                body=[
                    ast.parse(
                        f"states.append(({target_state.id},self.reader.save()))"
                    ).body[0],
                    ast.Continue(),
                ],
                orelse=[],
            )
            if not char.endswith("()"):
                trans_stmt.test = ast.parse(f"ch=={repr(char)}", mode="eval").body
                normal.append(trans_stmt)
                if target_state.id not in target_state_trans:
                    target_state_trans[target_state.id] = []
                target_state_trans[target_state.id].append(trans_stmt)
            else:
                trans_stmt.test = ast.parse(f"self.{char[:-2]}(ch)", mode="eval").body
                func.append(trans_stmt)
        # 合并转移到相同状态的分支
        # 只针对normal优先级
        for target, stmts in target_state_trans.items():
            if len(stmts) <= 1:
                continue
            for i in stmts:
                normal.remove(i)
            # 将多个 ch==... 转换成 ch in (...)
            i.test = ast.parse(
                f"ch in ({','.join(map(repr,[i.test.comparators[0].value for i in stmts]))})",
                mode="eval",
            ).body
            normal.append(i)
        body.extend(normal)
        body.extend(func)

        body.append(ast.Break())

        match_expr.cases.append(ast.match_case(pattern=pattern, body=body))

    match_expr.cases.append(ast.match_case(pattern=ast.MatchAs(), body=[ast.Break()]))

    # 生成getNewToken函数
    get_new_token_func = ast.FunctionDef(
        name="getNewToken",
        args=ast.arguments(
            posonlyargs=[],
            args=[ast.arg(arg="self")],
            kwonlyargs=[],
            kw_defaults=[],
            defaults=[],
        ),
        body=[
            ast.parse("start_index=self.reader.save()").body[0],
            ast.parse("states=[(0,start_index)]").body[0],
            ast.While(
                test=ast.Constant(value=True),
                body=[
                    ast.parse("ch,loc=self.reader.next()").body[0],
                    match_expr,
                ],
                orelse=[],
            ),
        ],
        decorator_list=[],
    )

    # 生成最终的接受部分
    accept_match = ast.Match(
        subject=ast.Name(id="state", ctx=ast.Load()),
        cases=[],
    )

    for state in states:
        for letter in state.involvedpos:
            if letter.char == "pattern_end":
                break
        else:
            continue
        accept_match.cases.append(
            ast.match_case(
                pattern=ast.MatchValue(value=ast.Constant(value=state.id)),
                body=[
                    ast.parse("self.reader.restore(back_index)").body[0],
                    ast.Return(
                        value=ast.parse(
                            letter.action.replace("ERROR", "self.error"), mode="eval"
                        ).body
                    ),
                ],
            )
        )

    get_new_token_func.body.append(
        ast.While(
            test=ast.Name(id="states", ctx=ast.Load()),
            body=[
                ast.parse("state, back_index = states.pop()").body[0],
                ast.parse('text=""').body[0],
                ast.parse("location=Location([])").body[0],
                ast.parse(
                    """
for i in range(start_index, back_index):
    text += self.reader.hasread[i][0]
    location += self.reader.hasread[i][1]
"""
                ).body[0],
                accept_match,
            ],
            orelse=[],
        )
    )

    get_new_token_func.body.extend(
        [
            ast.parse("self.reader.restore(start_index)").body[0],
            ast.Return(value=ast.Constant(value=None)),
        ]
    )

    classdef = ast.ClassDef(
        name="Gen_" + class_name,
        bases=[ast.Name(id="LexerBase", ctx=ast.Load())],
        keywords=[],
        body=[get_new_token_func],
        decorator_list=[],
    )
    ast.fix_missing_locations(classdef)
    module = ast.Module(
        body=[
            ast.parse("from Lex import LexerBase").body[0],
            ast.parse("from copy import deepcopy").body[0],
        ],
        type_ignores=[],
    )
    module.body.extend(ast.parse(header).body)
    module.body.append(classdef)
    return module
