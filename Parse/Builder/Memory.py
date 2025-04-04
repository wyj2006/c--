# 完全参考 https://web.cs.ucla.edu/~todd/research/pepm08.pdf


from copy import deepcopy
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Callable, Optional, TypeVar, Union
from Basic import Error

if TYPE_CHECKING:
    from .ParserBase import ParserBase

_T = TypeVar("_T")


@dataclass
class Head:
    rule: str
    involved_set: set
    eval_set: set


@dataclass
class LR:
    seed: Any
    rule: str
    head: Head


@dataclass
class MemoEntry:
    ans: Union[LR, Any]
    pos: int


@dataclass
class ContextPackage:
    parser: "ParserBase"
    method: Callable[["ParserBase"], _T]
    args: tuple
    kwargs: dict


def make_key(rule: str, pos: int, cp: ContextPackage):
    return rule, pos, cp.args, tuple(cp.kwargs.keys()), tuple(cp.kwargs.values())


def apple_rule(rule: str, pos: int, cp: ContextPackage):
    self = cp.parser
    method = cp.method

    m: Optional[MemoEntry] = recall(rule, pos, cp)
    if m == None:
        lr = LR(None, rule, None)
        self._lr_stack.append(lr)
        m = MemoEntry(lr, pos)
        self._memo[make_key(rule, pos, cp)] = m
        ans = method(self, *cp.args, **cp.kwargs)
        self._lr_stack.pop()
        m.pos = self.save()
        if lr.head != None:
            lr.seed = ans
            return lr_answer(rule, pos, m, cp)
        else:
            m.ans = ans
            return ans
    else:
        self.restore(m.pos)
        if isinstance(m.ans, LR):
            setup_lr(rule, m.ans, cp)
            return m.ans.seed
        else:
            return m.ans


def setup_lr(rule: str, lr: LR, cp: ContextPackage):
    self = cp.parser

    if lr.head == None:
        lr.head = Head(rule, set(), set())
    for s in self._lr_stack[::-1]:
        if s.head == lr.head:
            break
        lr.head.involved_set.add(s.rule)


def lr_answer(rule: str, pos: int, m: MemoEntry, cp: ContextPackage):
    h = m.ans.head
    if h.rule != rule:
        return m.ans.seed
    else:
        m.ans = m.ans.seed
        if m.ans == None:
            return None
        else:
            return grow_lr(rule, pos, m, h, cp)


def recall(rule: str, pos: int, cp: ContextPackage):
    self = cp.parser
    method = cp.method

    m: Optional[MemoEntry] = self._memo.get(make_key(rule, pos, cp), None)
    h: Head = self._heads.get(pos, None)
    if h == None:
        return m
    if m == None and rule not in h.involved_set:
        return MemoEntry(None, pos)
    if rule in h.eval_set:
        h.eval_set.remove(rule)
        ans = method(self, *cp.args, **cp.kwargs)
        m.ans = ans
        m.pos = self.save()
    return m


def grow_lr(rule: str, pos: int, m: MemoEntry, h: Head, cp: ContextPackage):
    self = cp.parser
    method = cp.method

    self._heads[pos] = h
    while True:
        self.restore(pos)
        h.eval_set = deepcopy(h.involved_set)
        ans = method(self, *cp.args, **cp.kwargs)
        if ans == None or self.save() <= m.pos:
            break
        m.ans = ans
        m.pos = self.save()
    self._heads.pop(pos)
    self.restore(m.pos)
    return m.ans


def memorize(method: Callable[["ParserBase"], _T]):
    rule = method.__name__

    def wrapper(self: "ParserBase", *args, **kwargs) -> _T:
        return apple_rule(
            rule,
            self.save(),
            ContextPackage(self, method, args=args, kwargs=kwargs),
        )

    return wrapper


memorize_left_rec = memorize

# 这段代码是不正确的
"""
def memorize(method: Callable[["ParserBase"], _T]):
    rule = method.__name__

    def wrapper(self: "ParserBase", *args, **kwargs) -> _T:
        pos = self.save()  # 当前位置
        key = rule, pos, args, tuple(kwargs.keys()), tuple(kwargs.values())
        if key not in self._memo:
            self._memo[key] = method(self, *args, **kwargs), self.save()
        ans, p = self._memo[key]
        self.restore(p)  # 匹配之后的位置
        return ans

    return wrapper


def memorize_left_rec(method: Callable[["ParserBase"], _T]):
    rule = method.__name__

    def wrapper(self: "ParserBase", *args, **kwargs) -> _T:
        pos = self.save()  # 当前位置
        key = rule, pos, args, tuple(kwargs.keys()), tuple(kwargs.values())
        if key not in self._memo:
            self._memo[key] = None, pos
            seed = method(self, *args, **kwargs)
            self._memo[key] = seed, self.save()
            while True:
                self.restore(pos)
                try:
                    ans = method(self, *args, **kwargs)
                except Error:
                    ans = None
                if ans == None or self.save() <= self._memo[key][1]:
                    break
                self._memo[key] = ans, self.save()
        ans, p = self._memo[key]
        self.restore(p)  # 匹配之后的位置
        return ans

    return wrapper
"""
