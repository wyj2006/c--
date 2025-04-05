from Parse.Builder import ParserBase, memorize, memorize_left_rec
from Basic import *

def check_keyword(a):
    if a.text in Token.keywords:
        a.kind = Token.keywords[a.text]
    return a

def check_punctuator(a, **kwargs):
    token = Token(Token.punctuator[a[1]], a[0], a[1])
    for key, val in kwargs.items():
        setattr(token, key, val)
    return token

def combine(*args):
    res = [Location([]), '']
    for i in args:
        if i == None:
            continue
        if isinstance(i, Token):
            i = (i.location, i.text)
        res[0] += i[0]
        res[1] += i[1]
    return tuple(res)

class Gen_GrammarLexerParser(ParserBase):

    @memorize
    def start(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (header := self.header()):
            return header
        self.restore(_z)
        if (action := self.action()):
            return action
        self.restore(_z)
        if (preprocessing_token := self.preprocessing_token()):
            return preprocessing_token
        self.restore(_z)
        if (token := self.token()):
            return token
        self.restore(_z)
        return None

    @memorize
    def header(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='"')) and (b := self.expect(TokenKind.IDENTIFIER, text='"')) and (c := self.expect(TokenKind.IDENTIFIER, text='"')) and (d := self.header_chars()) and (e := self.expect(TokenKind.IDENTIFIER, text='"')) and (f := self.expect(TokenKind.IDENTIFIER, text='"')) and (g := self.expect(TokenKind.IDENTIFIER, text='"')):
            return Token(TokenKind.HEADER, combine(a, b, c, d, e, f, g)[0], d.text)
        self.restore(_z)
        return None

    @memorize
    def action(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.L_BRACE)) and (b := self.action_chars()) and (c := self.expect(TokenKind.R_BRACE)):
            return Token(TokenKind.ACTION, combine(a, b, c)[0], b.text)
        self.restore(_z)
        return None

    @memorize
    def token(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.string_literal()):
            return Token(TokenKind.STRINGLITERAL, a[0], a[1])
        self.restore(_z)
        if (a := self.identifier()):
            return check_keyword(a)
        self.restore(_z)
        if (constant := self.constant()):
            return constant
        self.restore(_z)
        if (punctuator := self.punctuator()):
            return punctuator
        self.restore(_z)
        if (end := self.end()):
            return end
        self.restore(_z)
        return None

    @memorize_left_rec
    def identifier(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.identifier()) and (b := self.identifier_continue()):
            return Token(TokenKind.IDENTIFIER, a.location + b.location, a.text + b.text)
        self.restore(_z)
        if (identifier_start := self.identifier_start()):
            return identifier_start
        self.restore(_z)
        return None

    @memorize
    def universal_character_name(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='u')) and (c := self.hex_quad()):
            return Token(TokenKind.IDENTIFIER, a.location + b.location + c[0], a.text + b.text + c[1])
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='U')) and (c := self.hex_quad()) and (d := self.hex_quad()):
            return Token(TokenKind.IDENTIFIER, a.location + b.location + c[0] + d[0], a.text + b.text + c[1] + d[1])
        self.restore(_z)
        return None

    @memorize
    def hex_quad(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.hexadecimal_digit()) and (b := self.hexadecimal_digit()) and (c := self.hexadecimal_digit()) and (d := self.hexadecimal_digit()):
            return (a.location + b.location + c.location.d.location, a.text + b.text + c.text + d.text)
        self.restore(_z)
        return None

    @memorize
    def constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.floating_constant()):
            return Token(TokenKind.FLOATCONST, a[0], a[1])
        self.restore(_z)
        if (a := self.integer_constant()):
            return Token(TokenKind.INTCONST, a[0], a[1])
        self.restore(_z)
        if (a := self.character_constant()):
            return Token(TokenKind.CHARCONST, a[0], a[1])
        self.restore(_z)
        return None

    @memorize
    def integer_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.hexadecimal_constant()) and ((b := self.integer_suffix()),):
            return combine(a, b)
        self.restore(_z)
        if (a := self.binary_constant()) and ((b := self.integer_suffix()),):
            return combine(a, b)
        self.restore(_z)
        if (a := self.octal_constant()) and ((b := self.integer_suffix()),):
            return combine(a, b)
        self.restore(_z)
        if (a := self.decimal_constant()) and ((b := self.integer_suffix()),):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize_left_rec
    def decimal_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.decimal_constant()) and (self.expect(TokenKind.IDENTIFIER, text="'"),) and (b := self.digit()):
            return combine(a, b) + ("'",)
        self.restore(_z)
        if (a := self.nonzero_digit()):
            return (a.location, a.text)
        self.restore(_z)
        return None

    @memorize_left_rec
    def octal_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.octal_constant()) and (self.expect(TokenKind.IDENTIFIER, text="'"),) and (b := self.octal_digit()):
            return combine(a, b) + ("'",)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='0')):
            return (a.location, a.text)
        self.restore(_z)
        return None

    @memorize
    def hexadecimal_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.hexadecimal_prefix()) and (b := self.hexadecimal_digit_sequence()):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize_left_rec
    def binary_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.binary_constant()) and (self.expect(TokenKind.IDENTIFIER, text="'"),) and (b := self.binary_digit()):
            return combine(a, b) + ("'",)
        self.restore(_z)
        if (a := self.binary_prefix()) and (b := self.binary_digit()):
            return (a[0] + b.location, a[1] + b.text)
        self.restore(_z)
        return None

    @memorize
    def hexadecimal_prefix(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='0')) and (b := self.expect(TokenKind.IDENTIFIER, text='x')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='0')) and (b := self.expect(TokenKind.IDENTIFIER, text='X')):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize
    def binary_prefix(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='0')) and (b := self.expect(TokenKind.IDENTIFIER, text='b')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='0')) and (b := self.expect(TokenKind.IDENTIFIER, text='B')):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize_left_rec
    def hexadecimal_digit_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.hexadecimal_digit_sequence()) and (self.expect(TokenKind.IDENTIFIER, text="'"),) and (b := self.hexadecimal_digit()):
            return combine(a, b) + ("'",)
        self.restore(_z)
        if (a := self.hexadecimal_digit()):
            return (a.location, a.text)
        self.restore(_z)
        return None

    @memorize
    def integer_suffix(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.unsigned_suffix()) and (b := self.long_long_suffix()):
            return combine(a, b)
        self.restore(_z)
        if (a := self.unsigned_suffix()) and ((b := self.long_suffix()),):
            return combine(a, b)
        self.restore(_z)
        if (a := self.unsigned_suffix()) and (b := self.bit_precise_int_suffix()):
            return combine(a, b)
        self.restore(_z)
        if (a := self.long_long_suffix()) and ((b := self.unsigned_suffix()),):
            return combine(a, b)
        self.restore(_z)
        if (a := self.long_suffix()) and ((b := self.unsigned_suffix()),):
            return combine(a, b)
        self.restore(_z)
        if (a := self.bit_precise_int_suffix()) and ((b := self.unsigned_suffix()),):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize
    def bit_precise_int_suffix(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='w')) and (b := self.expect(TokenKind.IDENTIFIER, text='b')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='W')) and (b := self.expect(TokenKind.IDENTIFIER, text='B')):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize
    def long_long_suffix(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='l')) and (b := self.expect(TokenKind.IDENTIFIER, text='l')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='L')) and (b := self.expect(TokenKind.IDENTIFIER, text='L')):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize
    def floating_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (hexadecimal_floating_constant := self.hexadecimal_floating_constant()):
            return hexadecimal_floating_constant
        self.restore(_z)
        if (decimal_floating_constant := self.decimal_floating_constant()):
            return decimal_floating_constant
        self.restore(_z)
        return None

    @memorize
    def decimal_floating_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.fractional_constant()) and ((b := self.exponent_part()),) and ((c := self.floating_suffix()),):
            return combine(a, b, c)
        self.restore(_z)
        if (a := self.digit_sequence()) and (b := self.exponent_part()) and ((c := self.floating_suffix()),):
            return combine(a, b, c)
        self.restore(_z)
        return None

    @memorize
    def hexadecimal_floating_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.hexadecimal_prefix()) and (b := self.hexadecimal_fractional_constant()) and (c := self.binary_exponent_part()) and ((d := self.floating_suffix()),):
            return combine(a, b, c, d)
        self.restore(_z)
        if (a := self.hexadecimal_prefix()) and (b := self.hexadecimal_digit_sequence()) and (c := self.binary_exponent_part()) and ((d := self.floating_suffix()),):
            return combine(a, b, c, d)
        self.restore(_z)
        return None

    @memorize
    def fractional_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.digit_sequence()),) and (b := self.expect(TokenKind.PERIOD)) and (c := self.digit_sequence()):
            return combine(a, b, c)
        self.restore(_z)
        if (a := self.digit_sequence()) and (b := self.expect(TokenKind.PERIOD)):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize
    def exponent_part(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='e')) and ((b := self.sign()),) and (c := self.digit_sequence()):
            return combine(a, b, c)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='E')) and ((b := self.sign()),) and (c := self.digit_sequence()):
            return combine(a, b, c)
        self.restore(_z)
        return None

    @memorize
    def sign(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.PLUS)):
            return combine(a)
        self.restore(_z)
        if (a := self.expect(TokenKind.MINUS)):
            return combine(a)
        self.restore(_z)
        return None

    @memorize_left_rec
    def digit_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.digit_sequence()) and (self.expect(TokenKind.IDENTIFIER, text="'"),) and (b := self.digit()):
            return combine(a, b) + ("'",)
        self.restore(_z)
        if (a := self.digit()):
            return (a.location, a.text)
        self.restore(_z)
        return None

    @memorize
    def hexadecimal_fractional_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.hexadecimal_digit_sequence()),) and (b := self.expect(TokenKind.PERIOD)) and (c := self.hexadecimal_digit_sequence()):
            return combine(a, b, c)
        self.restore(_z)
        if (a := self.hexadecimal_digit_sequence()) and (b := self.expect(TokenKind.PERIOD)):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize
    def binary_exponent_part(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='p')) and ((b := self.sign()),) and (c := self.digit_sequence()):
            return combine(a, b, c)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='P')) and ((b := self.sign()),) and (c := self.digit_sequence()):
            return combine(a, b, c)
        self.restore(_z)
        return None

    @memorize
    def floating_suffix(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='d')) and (b := self.expect(TokenKind.IDENTIFIER, text='f')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='d')) and (b := self.expect(TokenKind.IDENTIFIER, text='d')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='d')) and (b := self.expect(TokenKind.IDENTIFIER, text='l')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='D')) and (b := self.expect(TokenKind.IDENTIFIER, text='F')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='D')) and (b := self.expect(TokenKind.IDENTIFIER, text='D')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='D')) and (b := self.expect(TokenKind.IDENTIFIER, text='L')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='f')):
            return combine(a)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='l')):
            return combine(a)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='F')):
            return combine(a)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='L')):
            return combine(a)
        self.restore(_z)
        return None

    @memorize
    def character_constant(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.encoding_prefix()),) and (b := self.expect(TokenKind.IDENTIFIER, text="'")) and (c := self.c_char_sequence()) and (d := self.expect(TokenKind.IDENTIFIER, text="'")):
            return combine(a, b, c, d)
        self.restore(_z)
        return None

    @memorize
    def encoding_prefix(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='u')) and (b := self.expect(TokenKind.IDENTIFIER, text='8')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='u')):
            return combine(a)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='U')):
            return combine(a)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='L')):
            return combine(a)
        self.restore(_z)
        return None

    @memorize_left_rec
    def c_char_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.c_char_sequence()) and (b := self.c_char()):
            return combine(a, b)
        self.restore(_z)
        if (a := self.c_char()):
            return combine(a)
        self.restore(_z)
        return None

    @memorize
    def escape_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (octal_escape_sequence := self.octal_escape_sequence()):
            return octal_escape_sequence
        self.restore(_z)
        if (hexadecimal_escape_sequence := self.hexadecimal_escape_sequence()):
            return hexadecimal_escape_sequence
        self.restore(_z)
        if (universal_character_name := self.universal_character_name()):
            return universal_character_name
        self.restore(_z)
        if (simple_escape_sequence := self.simple_escape_sequence()):
            return simple_escape_sequence
        self.restore(_z)
        return None

    @memorize
    def simple_escape_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text="'")):
            return combine(a, b) + "'"
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='"')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.QUESTION)):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='\\')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='a')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='b')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='f')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='n')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='r')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='t')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='v')):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')):
            return self.error('未知的转义字符', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize
    def octal_escape_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.octal_digit()) and (c := self.octal_digit()) and (d := self.octal_digit()):
            return combine(a, b, c, d)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.octal_digit()) and (c := self.octal_digit()):
            return combine(a, b, c)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.octal_digit()):
            return combine(a, b)
        self.restore(_z)
        return None

    @memorize_left_rec
    def hexadecimal_escape_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.hexadecimal_escape_sequence()) and (b := self.hexadecimal_digit()):
            return combine(a, b)
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\\')) and (b := self.expect(TokenKind.IDENTIFIER, text='x')) and (c := self.hexadecimal_digit()):
            return combine(a, b, c)
        self.restore(_z)
        return None

    @memorize
    def string_literal(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if ((a := self.encoding_prefix()),) and (b := self.expect(TokenKind.IDENTIFIER, text='"')) and ((c := self.s_char_sequence()),) and (d := self.expect(TokenKind.IDENTIFIER, text='"')):
            return combine(a, b, c, d)
        self.restore(_z)
        if ((a := self.encoding_prefix()),) and (b := self.expect(TokenKind.IDENTIFIER, text='"')) and ((c := self.s_char_sequence()),):
            return self.error('字符串未终止', self.curtoken().location)
        self.restore(_z)
        return None

    @memorize_left_rec
    def s_char_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.s_char_sequence()) and (b := self.s_char()):
            return combine(a, b)
        self.restore(_z)
        if (a := self.s_char()):
            return combine(a)
        self.restore(_z)
        return None

    @memorize
    def punctuator(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.PERIOD)) and (b := self.expect(TokenKind.PERIOD)) and (c := self.expect(TokenKind.PERIOD)):
            return check_punctuator(combine(a, b, c))
        self.restore(_z)
        if (a := self.expect(TokenKind.LESS)) and (b := self.expect(TokenKind.LESS)) and (c := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b, c))
        self.restore(_z)
        if (a := self.expect(TokenKind.GREATER)) and (b := self.expect(TokenKind.GREATER)) and (c := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b, c))
        self.restore(_z)
        if (a := self.expect(TokenKind.MINUS)) and (b := self.expect(TokenKind.GREATER)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.PLUS)) and (b := self.expect(TokenKind.PLUS)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.MINUS)) and (b := self.expect(TokenKind.MINUS)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.LESS)) and (b := self.expect(TokenKind.LESS)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.GREATER)) and (b := self.expect(TokenKind.GREATER)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.LESS)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.GREATER)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.EQUAL)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.EXCLAIM)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.AMP)) and (b := self.expect(TokenKind.AMP)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.PIPE)) and (b := self.expect(TokenKind.PIPE)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.COLON)) and (b := self.expect(TokenKind.COLON)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.STAR)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.SLASH)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.PERCENT)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.PLUS)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.MINUS)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.AMP)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.CARET)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.PIPE)) and (b := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.HASH)) and (b := self.expect(TokenKind.HASH)):
            return check_punctuator(combine(a, b))
        self.restore(_z)
        if (a := self.expect(TokenKind.L_SQUARE)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.R_SQUARE)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.L_PAREN)):
            return check_punctuator(combine(a), islparen=a.islparen)
        self.restore(_z)
        if (a := self.expect(TokenKind.R_PAREN)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.L_BRACE)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.R_BRACE)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.PERIOD)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.AMP)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.STAR)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.PLUS)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.MINUS)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.TILDE)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.EXCLAIM)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.SLASH)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.PERCENT)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.LESS)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.GREATER)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.CARET)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.PIPE)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.QUESTION)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.COLON)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.SEMI)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.EQUAL)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.COMMA)):
            return check_punctuator(combine(a))
        self.restore(_z)
        if (a := self.expect(TokenKind.HASH)):
            return check_punctuator(combine(a), ispphash=a.ispphash)
        self.restore(_z)
        return None

    @memorize
    def preprocessing_token(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.string_literal()):
            return Token(TokenKind.STRINGLITERAL, a[0], a[1])
        self.restore(_z)
        if (header_name := self.header_name()):
            return header_name
        self.restore(_z)
        if (comment := self.comment()):
            return comment
        self.restore(_z)
        if (a := self.expect(TokenKind.IDENTIFIER, text='\n')):
            return Token(TokenKind.NEWLINE, a.location, a.text)
        self.restore(_z)
        return None

    @memorize
    def header_name(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.LESS)) and (b := self.h_char_sequence()) and (c := self.expect(TokenKind.GREATER)):
            return Token(TokenKind.HEADERNAME, combine(a, b, c)[0], b[1])
        self.restore(_z)
        return None

    @memorize_left_rec
    def h_char_sequence(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.h_char_sequence()) and (b := self.h_char()):
            return combine(a, b)
        self.restore(_z)
        if (a := self.h_char()):
            return (a.location, a.text)
        self.restore(_z)
        return None

    @memorize
    def comment(self):
        begin_location = self.curtoken().location
        _z = self.save()
        if (a := self.expect(TokenKind.SLASH)) and (b := self.expect(TokenKind.SLASH)) and (c := self.single_line_comment()) and (d := self.expect(TokenKind.IDENTIFIER, text='\n')):
            return Token(TokenKind.COMMENT, combine(a, b, c, d)[0], c[1])
        self.restore(_z)
        if (a := self.expect(TokenKind.SLASH)) and (b := self.expect(TokenKind.STAR)) and (c := self.multi_line_comment()) and (d := self.expect(TokenKind.STAR)) and (e := self.expect(TokenKind.SLASH)):
            return Token(TokenKind.COMMENT, combine(a, b, c, d, e)[0], c[1])
        self.restore(_z)
        return None