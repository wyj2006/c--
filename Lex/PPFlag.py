from enum import IntEnum


class PPFlag(IntEnum):
    """预处理器状态"""

    KEEP_NEWLINE = 1  # 保留换行符
    ALLOW_REPLACE = 1 << 1  # 允许宏替换
    ALLOW_HEADERNAME = 1 << 2  # 解析 HEADERNAME
    KEEP_COMMENT = 1 << 3  # 保留注释
    IGNORE_PPDIRECTIVE = 1 << 4  # 忽略预处理指令
    ALLOW_CONTACT = 1 << 5  # 允许字符串拼接
    TRANS_PPKEYWORD = 1 << 6  # 将标识符转换成预处理关键字
