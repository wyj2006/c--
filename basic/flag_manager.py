class FlagManager:
    def __init__(self, flag: int):
        self.flag: int = flag

    def add(self, flag: int):
        self.flag |= flag

    def remove(self, flag: int):
        self.flag &= ~flag

    def has(self, flag: int):
        return self.flag & flag == flag

    def save(self):
        return self.flag

    def restore(self, flag: int):
        self.flag = flag
