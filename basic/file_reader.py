from typing import Literal
from basic import Error, Location


class FileReader:
    """文件读取器"""

    def __init__(self, filename: str, mode: Literal["r", "rb"] = "r"):
        self.filename = filename
        try:
            assert mode == "r" or mode == "rb"
            if mode == "r":
                self.lines = open(filename, encoding="utf-8").readlines()
            elif mode == "rb":
                content = open(filename, mode="rb").read()
                self.lines = []
                # 16个字节为一行
                for i in range(0, len(content), 16):
                    self.lines.append(content[i : i + 16])
        except FileNotFoundError:
            raise Error(f"无法打开文件: {filename}", Location())
        self.row = 0  # 从0开始
        self.col = 0  # 从0开始
        self.hasread: list[tuple[str, Location]] = []  # 已经读到的字符
        self.nextindex = 0
        if self.filename not in Location.lines:
            Location.lines[self.filename] = self.lines

    def current(self) -> tuple[str, Location]:
        return self.hasread[self.nextindex - 1]

    def next(self) -> tuple[str, Location]:
        """读取一个字符, 并返回这个字符和它对应的片段"""
        if self.nextindex >= len(self.hasread):
            location = Location(
                [
                    {
                        "filename": self.filename,
                        "lineno": self.row + 1,
                        "col": self.col + 1,
                        "span_col": 1,
                    }
                ]
            )
            # self.row和self.col永远指向下一个字符位置
            if self.row >= len(self.lines):  # 文件结束
                self.hasread.append(("", location))
            else:
                ch = self.lines[self.row][self.col]
                self.col += 1
                if self.col >= len(self.lines[self.row]):
                    self.row += 1
                    self.col = 0
                self.hasread.append((ch, location))
        ch, location = self.hasread[self.nextindex]
        self.nextindex += 1
        return ch, location

    def back(self):
        """回退当前已经读到的字符, 使下一次读取时重新读到这个字符"""
        self.nextindex -= 1

    def save(self):
        return self.nextindex

    def restore(self, index):
        self.nextindex = index


class MergeReader(FileReader):
    """
    凡在反斜杠出现于行尾（紧跟换行符）时，
    删除反斜杠和换行符，
    把两个物理源码行组合成一个逻辑源码行
    """

    def next(self):
        char, location = super().next()
        while char == "\\":
            ch, _ = super().next()
            if ch == "\n":
                for i in range(2):
                    self.nextindex -= 1
                    self.hasread.pop(self.nextindex)
                char, location = super().next()
            else:
                self.back()
                break
        return char, location
