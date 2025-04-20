from AST import Letter


class State:
    """状态"""

    def __init__(self, id: int, involvedpos: set[Letter]):
        self.id = id  # 编号
        self.involvedpos: set[Letter] = involvedpos  # 包含的字符
        self.transition: dict[str, State] = {}  # 状态转移

    def __eq__(self, other):
        return isinstance(other, State) and self.involvedpos == other.involvedpos
