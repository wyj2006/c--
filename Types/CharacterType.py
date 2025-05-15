from Types.Type import Type


class CharacterType(Type):
    pass


class CharType(CharacterType):
    is_integer_type = True
    is_arithmetic_type = True
    is_real_type = True
    is_scalar_type = True

    size = 1

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="char"))


class SCharType(CharacterType):
    size = 1

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="signed char"))


class UCharType(CharacterType):
    size = 1

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name="unsigned char")
        )
