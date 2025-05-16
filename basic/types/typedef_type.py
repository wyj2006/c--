from .ctype import Type
from basic import Symbol


class TypedefType(Type, Symbol):
    def __init__(self, name: str, type: Type):
        super().__init__()
        self.name = name
        self.type = type

    def __str__(self):
        return f"{self.name}:{self.type}"

    def genDeclaration(self, declaration):
        from basic import TypedefSpecifier

        declaration.specifiers.append(TypedefSpecifier(specifier_name=self.name))

    def __eq__(self, other):
        return (
            isinstance(other, TypedefType)
            and self.name == other.name
            and self.type == other.type
        )

    def is_complete(self):
        return self.type.is_complete()


class Char8Type(TypedefType):
    def __init__(self):
        from basic import CharType

        super().__init__("char8_t", CharType())


class Char16Type(TypedefType):
    def __init__(self):
        from basic import ShortType

        super().__init__("char16_t", ShortType())


class Char32Type(TypedefType):
    def __init__(self):
        from basic import LongType

        super().__init__("char32_t", LongType())


class WCharType(TypedefType):
    def __init__(self):
        from basic import IntType

        super().__init__("wchar_t", IntType())


class SizeType(TypedefType):
    def __init__(self):
        from basic import ULongType

        super().__init__("size_t", ULongType())
