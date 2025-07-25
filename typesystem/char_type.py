from .ctype import Type
from .integer_type import IntegerType


class CharacterType(Type):
    pass


class CharType(CharacterType, IntegerType):
    is_integer_type = True
    is_arithmetic_type = True
    is_real_type = True
    is_scalar_type = True

    size = 1
    signed = True

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="char"))


class SCharType(CharacterType):
    size = 1

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="signed char"))


class UCharType(CharacterType):
    size = 1

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="unsigned char")
        )
