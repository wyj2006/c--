# 完全参考 https://web.cs.ucla.edu/~todd/research/pepm08.pdf


from typing import TYPE_CHECKING, Callable, TypeVar
from basic import Error

if TYPE_CHECKING:
    from .parser_base import ParserBase

_T = TypeVar("_T")


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
