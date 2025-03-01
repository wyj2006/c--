from enum import IntEnum


class FlagManager:
    def __init__(self, flag: IntEnum):
        self.flag: int = flag

    def add(self, flag: IntEnum):
        self.flag |= flag

    def remove(self, flag: IntEnum):
        self.flag &= ~flag

    def has(self, flag: IntEnum):
        return self.flag & flag == flag

    def save(self):
        return self.flag

    def restore(self, flag: int):
        self.flag = flag
