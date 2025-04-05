from Basic import FileReader


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
