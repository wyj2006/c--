from lex import LexerBase
from copy import deepcopy
from basic import *

class Gen_Lexer(LexerBase):

    def get_new_token(self):
        start_index = self.reader.save()
        states = [(0, start_index)]
        while True:
            ch, loc = self.reader.next()
            match states[-1][0]:
                case 0:
                    if ch == '|':
                        states.append((1, self.reader.save()))
                        continue
                    if ch == ':':
                        states.append((3, self.reader.save()))
                        continue
                    if ch == '0':
                        states.append((4, self.reader.save()))
                        continue
                    if ch == '*':
                        states.append((5, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((6, self.reader.save()))
                        continue
                    if ch == '/':
                        states.append((7, self.reader.save()))
                        continue
                    if ch == '%':
                        states.append((8, self.reader.save()))
                        continue
                    if ch == '+':
                        states.append((9, self.reader.save()))
                        continue
                    if ch == '-':
                        states.append((11, self.reader.save()))
                        continue
                    if ch == '&':
                        states.append((12, self.reader.save()))
                        continue
                    if ch == '^':
                        states.append((13, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((14, self.reader.save()))
                        continue
                    if ch == '#':
                        states.append((15, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((18, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((19, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((20, self.reader.save()))
                        continue
                    if ch == '<':
                        states.append((21, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((22, self.reader.save()))
                        continue
                    if ch == '!':
                        states.append((23, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((24, self.reader.save()))
                        continue
                    if ch == '':
                        states.append((25, self.reader.save()))
                        continue
                    if ch in ('Y', 'i', 'Z', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'M', 'N', 'O', 'P', '_', 'Q', 'a', 'R', 'b', 'S', 'c', 'T', 'd', 'e', 'V', 'f', 'W', 'g', 'X', 'h'):
                        states.append((2, self.reader.save()))
                        continue
                    if ch in ('1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((10, self.reader.save()))
                        continue
                    if ch in ('U', 'L'):
                        states.append((16, self.reader.save()))
                        continue
                    if ch in ('[', ']', '(', ')', '{', '}', '~', '?', ';', ','):
                        states.append((17, self.reader.save()))
                        continue
                    if self.other_identifier_start(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 1:
                    if ch in ('|', '='):
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 2:
                    if ch == '\\':
                        states.append((27, self.reader.save()))
                        continue
                    if ch in ('y', 'T', 'd', 'z', 'U', 'e', 'A', 'V', 'f', 'B', 'W', 'g', 'C', 'X', 'h', 'D', 'Y', 'i', 'E', 'Z', 'j', '0', 'F', 'k', '1', 'G', 'l', '2', 'H', 'm', '3', 'I', 'n', '4', 'J', 'o', '5', 'K', 'p', '6', 'L', 'q', '7', 'M', 'r', '8', 'N', 's', '9', 'O', 't', 'P', 'u', '_', 'Q', 'v', 'a', 'R', 'w', 'b', 'S', 'x', 'c'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 3:
                    if ch == ':':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 4:
                    if ch == 'l':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((31, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((33, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((34, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((36, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((37, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((39, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((40, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((28, self.reader.save()))
                        continue
                    if ch in ('7', '1', '3', '0', '2', '4', '5', '6'):
                        states.append((29, self.reader.save()))
                        continue
                    if ch in ('b', 'B'):
                        states.append((32, self.reader.save()))
                        continue
                    if ch in ('x', 'X'):
                        states.append((35, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((38, self.reader.save()))
                        continue
                    break
                case 5:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 6:
                    if ch == 'U':
                        states.append((41, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((42, self.reader.save()))
                        continue
                    break
                case 7:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 8:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 9:
                    if ch in ('=', '+'):
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 10:
                    if ch == 'e':
                        states.append((31, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((44, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((45, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((46, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((48, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((39, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((40, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((43, self.reader.save()))
                        continue
                    if ch in ('8', '1', '3', '0', '9', '2', '4', '5', '6', '7'):
                        states.append((10, self.reader.save()))
                        continue
                    break
                case 11:
                    if ch in ('-', '>', '='):
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 12:
                    if ch in ('&', '='):
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 13:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 14:
                    if ch == '8':
                        states.append((16, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((19, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((27, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((18, self.reader.save()))
                        continue
                    if ch in ('y', 'T', 'd', 'z', 'U', 'e', 'A', 'V', 'f', 'B', 'W', 'g', 'C', 'X', 'h', 'D', 'Y', 'i', 'E', 'Z', 'j', '0', 'F', 'k', '1', 'G', 'l', '2', 'H', 'm', '3', 'I', 'n', '4', 'J', 'o', '5', 'K', 'p', '6', 'L', 'q', '7', 'M', 'r', 'N', 's', '9', 'O', 't', 'P', 'u', '_', 'Q', 'v', 'a', 'R', 'w', 'b', 'S', 'x', 'c'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 15:
                    if ch == '#':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 16:
                    if ch == '"':
                        states.append((19, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((27, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((18, self.reader.save()))
                        continue
                    if ch in ('y', 'T', 'd', 'z', 'U', 'e', 'A', 'V', 'f', 'B', 'W', 'g', 'C', 'X', 'h', 'D', 'Y', 'i', 'E', 'Z', 'j', '0', 'F', 'k', '1', 'G', 'l', '2', 'H', 'm', '3', 'I', 'n', '4', 'J', 'o', '5', 'K', 'p', '6', 'L', 'q', '7', 'M', 'r', '8', 'N', 's', '9', 'O', 't', 'P', 'u', '_', 'Q', 'v', 'a', 'R', 'w', 'b', 'S', 'x', 'c'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 17:
                    break
                case 18:
                    if ch == '\\':
                        states.append((49, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 19:
                    if ch == '\\':
                        states.append((51, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 20:
                    if ch == '.':
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('1', '4', '9', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 21:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    if ch == '<':
                        states.append((56, self.reader.save()))
                        continue
                    break
                case 22:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 23:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 24:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 25:
                    break
                case 26:
                    break
                case 27:
                    if ch == 'u':
                        states.append((58, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 28:
                    if ch == 'l':
                        states.append((60, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((61, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((62, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((63, self.reader.save()))
                        continue
                    break
                case 29:
                    if ch == 'l':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((31, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((33, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((34, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((36, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((37, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((39, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((40, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((28, self.reader.save()))
                        continue
                    if ch in ('7', '3', '1', '0', '4', '2', '5', '6'):
                        states.append((29, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((38, self.reader.save()))
                        continue
                    break
                case 30:
                    if ch == 'l':
                        states.append((65, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 31:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((66, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 32:
                    if ch in ('1', '0'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 33:
                    if ch in ('7', '3', '1', '0', '4', '2', '5', '6'):
                        states.append((29, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((38, self.reader.save()))
                        continue
                    break
                case 34:
                    if ch == 'L':
                        states.append((65, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 35:
                    if ch == '.':
                        states.append((70, self.reader.save()))
                        continue
                    if ch in ('a', '9', '2', 'f', 'b', '3', 'A', 'c', '4', 'B', 'd', '5', 'C', 'e', '6', 'D', '7', 'E', '8', 'F', '0', '1'):
                        states.append((69, self.reader.save()))
                        continue
                    break
                case 36:
                    if ch == 'b':
                        states.append((71, self.reader.save()))
                        continue
                    break
                case 37:
                    if ch == 'B':
                        states.append((71, self.reader.save()))
                        continue
                    break
                case 38:
                    if ch == 'e':
                        states.append((31, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((72, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((39, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((40, self.reader.save()))
                        continue
                    if ch in ('8', '6', '3', '1', '5', '0', '9', '4', '7', '2'):
                        states.append((38, self.reader.save()))
                        continue
                    break
                case 39:
                    if ch == 'D':
                        states.append((73, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((75, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((76, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((77, self.reader.save()))
                        continue
                    if ch in ('4', '9', '5', '0', '3', '6', '1', '7', '2', '8'):
                        states.append((54, self.reader.save()))
                        continue
                    if ch in ('L', 'f', 'l', 'F'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 40:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((78, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((79, self.reader.save()))
                        continue
                    break
                case 41:
                    if ch in ('d', '8', 'C', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2'):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 42:
                    if ch in ('f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', 'E', '4'):
                        states.append((81, self.reader.save()))
                        continue
                    break
                case 43:
                    if ch == 'L':
                        states.append((82, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((83, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((84, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((85, self.reader.save()))
                        continue
                    break
                case 44:
                    if ch in ('8', '3', '1', '9', '0', '4', '2', '5', '6', '7'):
                        states.append((10, self.reader.save()))
                        continue
                    break
                case 45:
                    if ch == 'l':
                        states.append((86, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 46:
                    if ch == 'L':
                        states.append((86, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 47:
                    if ch == 'b':
                        states.append((87, self.reader.save()))
                        continue
                    break
                case 48:
                    if ch == 'B':
                        states.append((87, self.reader.save()))
                        continue
                    break
                case 49:
                    if ch == 'x':
                        states.append((89, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((90, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((91, self.reader.save()))
                        continue
                    if ch in ('3', '7', '4', '5', '0', '6', '1', '2'):
                        states.append((88, self.reader.save()))
                        continue
                    if ch in ('n', "'", 'r', '"', 't', '?', 'v', '\\', 'a', 'b', 'f'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 50:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 51:
                    if ch == 'u':
                        states.append((95, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('7', '4', '5', '6', '0', '1', '2', '3'):
                        states.append((94, self.reader.save()))
                        continue
                    if ch in ('r', "'", 't', '"', 'v', '?', '\\', 'a', 'b', 'f', 'n'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 52:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 53:
                    break
                case 54:
                    if ch == 'D':
                        states.append((73, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((75, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((76, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((77, self.reader.save()))
                        continue
                    if ch in ('1', '7', '2', '8', '3', '9', '4', '5', '0', '6'):
                        states.append((54, self.reader.save()))
                        continue
                    if ch in ('L', 'f', 'l', 'F'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 55:
                    if ch == '.':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 56:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 57:
                    if ch == '=':
                        states.append((17, self.reader.save()))
                        continue
                    break
                case 58:
                    if ch in ('a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', 'E', '3', 'D', 'e', '9', '4', 'f'):
                        states.append((100, self.reader.save()))
                        continue
                    break
                case 59:
                    if ch in ('8', '3', 'D', 'e', '9', '4', 'E', 'C', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'd'):
                        states.append((101, self.reader.save()))
                        continue
                    break
                case 60:
                    if ch == 'l':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 61:
                    if ch == 'L':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 62:
                    if ch == 'b':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 63:
                    if ch == 'B':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 64:
                    break
                case 65:
                    if ch in ('U', 'u'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 66:
                    if ch == 'd':
                        states.append((102, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((103, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((104, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((74, self.reader.save()))
                        continue
                    if ch in ('7', '2', '8', '3', '9', '4', '5', '0', '6', '1'):
                        states.append((66, self.reader.save()))
                        continue
                    break
                case 67:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((66, self.reader.save()))
                        continue
                    break
                case 68:
                    if ch == 'l':
                        states.append((106, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((107, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((108, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((109, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((110, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((105, self.reader.save()))
                        continue
                    if ch in ('0', '1'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 69:
                    if ch == "'":
                        states.append((112, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((113, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((114, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((115, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((116, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((117, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((118, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((119, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((111, self.reader.save()))
                        continue
                    if ch in ('5', 'D', '7', 'b', '6', 'E', '8', 'c', '0', 'F', '9', 'd', '1', 'a', 'e', '2', 'f', '3', 'A', '4', 'B', 'C'):
                        states.append((69, self.reader.save()))
                        continue
                    break
                case 70:
                    if ch in ('5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', 'E', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'f', 'a'):
                        states.append((120, self.reader.save()))
                        continue
                    break
                case 71:
                    if ch in ('u', 'U'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 72:
                    if ch in ('8', '6', '3', '1', '5', '9', '0', '4', '7', '2'):
                        states.append((38, self.reader.save()))
                        continue
                    break
                case 73:
                    if ch in ('L', 'F', 'D'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 74:
                    break
                case 75:
                    if ch in ('d', 'l', 'f'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 76:
                    if ch in ('1', '4', '9', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((121, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((122, self.reader.save()))
                        continue
                    break
                case 77:
                    if ch in ('6', '1', '9', '4', '7', '0', '2', '5', '8', '3'):
                        states.append((123, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((124, self.reader.save()))
                        continue
                    break
                case 78:
                    if ch == 'D':
                        states.append((103, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((102, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((125, self.reader.save()))
                        continue
                    if ch in ('4', '5', '0', '6', '1', '7', '2', '8', '3', '9'):
                        states.append((78, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 79:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 80:
                    if ch in ('1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6'):
                        states.append((126, self.reader.save()))
                        continue
                    break
                case 81:
                    if ch in ('e', '9', '4', 'E', 'D', 'f', 'a', '5', 'F', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3'):
                        states.append((127, self.reader.save()))
                        continue
                    break
                case 82:
                    if ch == 'L':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 83:
                    if ch == 'l':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 84:
                    if ch == 'b':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 85:
                    if ch == 'B':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 86:
                    if ch in ('U', 'u'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 87:
                    if ch in ('u', 'U'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 88:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('5', '0', '6', '1', '7', '2', '3', '4'):
                        states.append((128, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 89:
                    if ch in ('0', 'A', 'b', 'F', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5'):
                        states.append((129, self.reader.save()))
                        continue
                    break
                case 90:
                    if ch in ('1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'A', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'b', '6'):
                        states.append((130, self.reader.save()))
                        continue
                    break
                case 91:
                    if ch in ('3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8'):
                        states.append((131, self.reader.save()))
                        continue
                    break
                case 92:
                    if ch == 'u':
                        states.append((132, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((134, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((135, self.reader.save()))
                        continue
                    if ch in ('3', '6', '2', '4', '7', '5', '0', '1'):
                        states.append((133, self.reader.save()))
                        continue
                    if ch in ('n', "'", 'r', '"', 't', '?', 'f', 'v', '\\', 'a', 'b'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 93:
                    break
                case 94:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('6', '5', '1', '7', '0', '2', '3', '4'):
                        states.append((136, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 95:
                    if ch in ('9', 'E', '8', 'D', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C'):
                        states.append((137, self.reader.save()))
                        continue
                    break
                case 96:
                    if ch in ('6', 'f', 'B', '5', '7', 'A', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4'):
                        states.append((138, self.reader.save()))
                        continue
                    break
                case 97:
                    if ch in ('0', 'b', '2', 'd', '1', 'c', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F'):
                        states.append((139, self.reader.save()))
                        continue
                    break
                case 98:
                    if ch == 'u':
                        states.append((141, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((142, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((143, self.reader.save()))
                        continue
                    if ch in ('6', '3', '7', '4', '5', '0', '1', '2'):
                        states.append((140, self.reader.save()))
                        continue
                    if ch in ('n', 'r', "'", 't', '"', 'v', '?', '\\', 'a', 'b', 'f'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 99:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 100:
                    if ch in ('9', '4', 'E', 'f', 'D', 'a', '5', 'F', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'e'):
                        states.append((144, self.reader.save()))
                        continue
                    break
                case 101:
                    if ch in ('c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', '1', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B'):
                        states.append((145, self.reader.save()))
                        continue
                    break
                case 102:
                    if ch in ('f', 'd', 'l'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 103:
                    if ch in ('F', 'D', 'L'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 104:
                    if ch in ('4', '7', '2', '5', '9', '0', '8', '3', '6', '1'):
                        states.append((66, self.reader.save()))
                        continue
                    break
                case 105:
                    if ch == 'l':
                        states.append((146, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((147, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((148, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((149, self.reader.save()))
                        continue
                    break
                case 106:
                    if ch == 'l':
                        states.append((150, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 107:
                    if ch == 'L':
                        states.append((150, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 108:
                    if ch in ('0', '1'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 109:
                    if ch == 'b':
                        states.append((151, self.reader.save()))
                        continue
                    break
                case 110:
                    if ch == 'B':
                        states.append((151, self.reader.save()))
                        continue
                    break
                case 111:
                    if ch == 'L':
                        states.append((152, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((153, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((154, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((155, self.reader.save()))
                        continue
                    break
                case 112:
                    if ch in ('D', '5', '7', 'b', 'E', '6', '8', 'c', '0', 'F', '9', 'd', '1', 'a', 'e', '2', 'f', '3', 'A', '4', 'B', 'C'):
                        states.append((69, self.reader.save()))
                        continue
                    break
                case 113:
                    if ch in ('2', '5', '0', '8', '4', '3', '6', '1', '9', '7'):
                        states.append((156, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((157, self.reader.save()))
                        continue
                    break
                case 114:
                    if ch == 'p':
                        states.append((158, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((159, self.reader.save()))
                        continue
                    if ch in ('5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', 'E', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'f', 'a'):
                        states.append((120, self.reader.save()))
                        continue
                    break
                case 115:
                    if ch == 'b':
                        states.append((160, self.reader.save()))
                        continue
                    break
                case 116:
                    if ch == 'B':
                        states.append((160, self.reader.save()))
                        continue
                    break
                case 117:
                    if ch == 'l':
                        states.append((161, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 118:
                    if ch == 'L':
                        states.append((161, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 119:
                    if ch in ('4', '2', '5', '0', '8', '1', '3', '6', '9', '7'):
                        states.append((162, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((163, self.reader.save()))
                        continue
                    break
                case 120:
                    if ch == 'p':
                        states.append((158, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((164, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((159, self.reader.save()))
                        continue
                    if ch in ('c', '7', '2', 'C', 'B', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1'):
                        states.append((120, self.reader.save()))
                        continue
                    break
                case 121:
                    if ch == 'D':
                        states.append((73, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((75, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((165, self.reader.save()))
                        continue
                    if ch in ('1', '7', '2', '8', '3', '9', '4', '5', '0', '6'):
                        states.append((121, self.reader.save()))
                        continue
                    if ch in ('L', 'f', 'l', 'F'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 122:
                    if ch in ('1', '4', '9', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((121, self.reader.save()))
                        continue
                    break
                case 123:
                    if ch == 'D':
                        states.append((73, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((166, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((75, self.reader.save()))
                        continue
                    if ch in ('9', '4', '5', '0', '6', '1', '7', '2', '8', '3'):
                        states.append((123, self.reader.save()))
                        continue
                    if ch in ('L', 'f', 'l', 'F'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 124:
                    if ch in ('6', '1', '9', '4', '7', '2', '8', '5', '0', '3'):
                        states.append((123, self.reader.save()))
                        continue
                    break
                case 125:
                    if ch in ('4', '7', '2', '5', '0', '8', '3', '6', '9', '1'):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 126:
                    if ch in ('0', 'A', 'b', '6', '1', 'B', 'c', '7', 'F', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5'):
                        states.append((167, self.reader.save()))
                        continue
                    break
                case 127:
                    if ch in ('2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7'):
                        states.append((168, self.reader.save()))
                        continue
                    break
                case 128:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('7', '2', '3', '4', '0', '5', '1', '6'):
                        states.append((50, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 129:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd'):
                        states.append((129, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 130:
                    if ch in ('5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a'):
                        states.append((169, self.reader.save()))
                        continue
                    break
                case 131:
                    if ch in ('2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', 'c', '1', '7'):
                        states.append((170, self.reader.save()))
                        continue
                    break
                case 132:
                    if ch in ('8', '3', 'C', 'e', 'D', '9', '4', 'f', 'E', 'a', '5', '0', 'A', 'F', 'b', '6', '1', 'B', 'c', '7', '2', 'd'):
                        states.append((171, self.reader.save()))
                        continue
                    break
                case 133:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('5', '0', '6', '1', '7', '2', '3', '4'):
                        states.append((172, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 134:
                    if ch in ('a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', 'E', '9', '4', 'f'):
                        states.append((173, self.reader.save()))
                        continue
                    break
                case 135:
                    if ch in ('A', '6', 'b', '1', 'B', '7', 'c', '2', 'C', '8', 'd', '3', 'D', '9', 'e', '4', 'E', 'a', 'f', 'F', '5', '0'):
                        states.append((174, self.reader.save()))
                        continue
                    break
                case 136:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('1', '3', '2', '4', '5', '6', '7', '0'):
                        states.append((52, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 137:
                    if ch in ('8', 'B', 'D', '7', '9', 'C', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6'):
                        states.append((175, self.reader.save()))
                        continue
                    break
                case 138:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('e', '2', 'f', '3', 'A', '4', 'B', '5', 'C', '6', 'D', '7', 'E', '8', '0', 'F', '9', '1', 'a', 'b', 'c', 'd'):
                        states.append((138, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 139:
                    if ch in ('b', 'E', '1', 'a', 'c', '0', 'F', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9'):
                        states.append((176, self.reader.save()))
                        continue
                    break
                case 140:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('5', '0', '4', '6', '1', '7', '2', '3'):
                        states.append((177, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 141:
                    if ch in ('D', '7', 'C', '9', 'E', '8', 'a', '0', 'F', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B'):
                        states.append((178, self.reader.save()))
                        continue
                    break
                case 142:
                    if ch in ('A', '4', 'f', '6', 'B', '5', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e'):
                        states.append((179, self.reader.save()))
                        continue
                    break
                case 143:
                    if ch in ('a', '1', 'F', 'c', '0', 'b', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E'):
                        states.append((180, self.reader.save()))
                        continue
                    break
                case 144:
                    if ch in ('d', '8', '3', 'D', 'e', 'C', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2'):
                        states.append((181, self.reader.save()))
                        continue
                    break
                case 145:
                    if ch in ('b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', 'F', '9', '4', 'E', 'f', '0', 'a', '5', 'A'):
                        states.append((182, self.reader.save()))
                        continue
                    break
                case 146:
                    if ch == 'l':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 147:
                    if ch == 'L':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 148:
                    if ch == 'b':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 149:
                    if ch == 'B':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 150:
                    if ch in ('u', 'U'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 151:
                    if ch in ('U', 'u'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 152:
                    if ch == 'L':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 153:
                    if ch == 'l':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 154:
                    if ch == 'b':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 155:
                    if ch == 'B':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 156:
                    if ch == 'D':
                        states.append((183, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((184, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((185, self.reader.save()))
                        continue
                    if ch in ('5', '0', '6', '1', '7', '2', '8', '3', '9', '4'):
                        states.append((156, self.reader.save()))
                        continue
                    if ch in ('l', 'f', 'F', 'L'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 157:
                    if ch in ('2', '5', '0', '8', '3', '6', '1', '9', '4', '7'):
                        states.append((156, self.reader.save()))
                        continue
                    break
                case 158:
                    if ch in ('-', '+'):
                        states.append((186, self.reader.save()))
                        continue
                    if ch in ('5', '0', '8', '3', '6', '1', '9', '4', '7', '2'):
                        states.append((187, self.reader.save()))
                        continue
                    break
                case 159:
                    if ch in ('0', '8', '3', '5', '6', '1', '9', '4', '7', '2'):
                        states.append((188, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 160:
                    if ch in ('U', 'u'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 161:
                    if ch in ('u', 'U'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 162:
                    if ch == 'D':
                        states.append((183, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((184, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((190, self.reader.save()))
                        continue
                    if ch in ('l', 'F', 'L', 'f'):
                        states.append((74, self.reader.save()))
                        continue
                    if ch in ('3', '8', '4', '9', '5', '0', '6', '1', '7', '2'):
                        states.append((162, self.reader.save()))
                        continue
                    break
                case 163:
                    if ch in ('2', '5', '0', '8', '3', '9', '6', '1', '4', '7'):
                        states.append((162, self.reader.save()))
                        continue
                    break
                case 164:
                    if ch in ('c', '7', '1', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B'):
                        states.append((120, self.reader.save()))
                        continue
                    break
                case 165:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((121, self.reader.save()))
                        continue
                    break
                case 166:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((123, self.reader.save()))
                        continue
                    break
                case 167:
                    if ch in ('9', '4', 'E', 'f', 'a', '0', '5', 'F', 'A', 'b', '1', '6', 'B', 'c', '2', '7', 'C', 'd', '3', '8', 'D', 'e'):
                        states.append((191, self.reader.save()))
                        continue
                    break
                case 168:
                    if ch in ('1', 'B', '7', 'c', '2', 'C', '8', 'd', 'F', '3', 'D', '9', 'e', '4', 'A', 'E', 'a', 'f', '5', '0', 'b', '6'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 169:
                    if ch in ('4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', 'D', '8', '3', 'e', '9'):
                        states.append((192, self.reader.save()))
                        continue
                    break
                case 170:
                    if ch in ('6', '1', 'B', 'c', '7', '0', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', 'A', 'b'):
                        states.append((193, self.reader.save()))
                        continue
                    break
                case 171:
                    if ch in ('1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6'):
                        states.append((194, self.reader.save()))
                        continue
                    break
                case 172:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('7', '2', '3', '1', '4', '5', '0', '6'):
                        states.append((50, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 173:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c'):
                        states.append((173, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 174:
                    if ch in ('f', 'a', 'E', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4'):
                        states.append((195, self.reader.save()))
                        continue
                    break
                case 175:
                    if ch in ('5', '7', 'A', 'C', '6', '8', 'B', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f'):
                        states.append((196, self.reader.save()))
                        continue
                    break
                case 176:
                    if ch in ('8', 'a', 'D', '0', 'F', '9', 'b', 'E', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C'):
                        states.append((197, self.reader.save()))
                        continue
                    break
                case 177:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('2', '0', '3', '1', '4', '5', '6', '7'):
                        states.append((52, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 178:
                    if ch in ('7', 'C', '6', 'B', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A'):
                        states.append((198, self.reader.save()))
                        continue
                    break
                case 179:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('1', 'e', '2', 'f', '3', 'A', '4', 'B', '5', 'C', '6', 'D', '7', 'E', '8', 'F', '0', '9', 'a', 'b', 'c', 'd'):
                        states.append((179, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 180:
                    if ch in ('F', '9', '0', 'E', 'b', 'a', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D'):
                        states.append((199, self.reader.save()))
                        continue
                    break
                case 181:
                    if ch in ('B', 'c', '6', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '4', '9', 'E', 'f', '5', 'a', '0', 'F', 'A', 'b', '1'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 182:
                    if ch in ('4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9'):
                        states.append((200, self.reader.save()))
                        continue
                    break
                case 183:
                    if ch in ('D', 'L', 'F'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 184:
                    if ch in ('f', 'd', 'l'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 185:
                    if ch in ('5', '0', '8', '3', '6', '1', '9', '4', '7', '2'):
                        states.append((156, self.reader.save()))
                        continue
                    break
                case 186:
                    if ch in ('5', '0', '8', '3', '6', '1', '9', '4', '7', '2'):
                        states.append((187, self.reader.save()))
                        continue
                    break
                case 187:
                    if ch == 'd':
                        states.append((201, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((202, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((203, self.reader.save()))
                        continue
                    if ch in ('0', '6', '1', '7', '2', '8', '3', '9', '4', '5'):
                        states.append((187, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 188:
                    if ch == 'd':
                        states.append((201, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((204, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((202, self.reader.save()))
                        continue
                    if ch in ('3', '9', '4', '5', '0', '6', '1', '7', '2', '8'):
                        states.append((188, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 189:
                    if ch in ('0', '8', '3', '6', '1', '9', '4', '7', '2', '5'):
                        states.append((188, self.reader.save()))
                        continue
                    break
                case 190:
                    if ch in ('5', '0', '3', '8', '6', '1', '9', '4', '7', '2'):
                        states.append((162, self.reader.save()))
                        continue
                    break
                case 191:
                    if ch in ('7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', '1', 'b', '6', 'B', 'c'):
                        states.append((205, self.reader.save()))
                        continue
                    break
                case 192:
                    if ch in ('d', '3', '8', 'D', 'e', '4', '9', 'E', 'f', 'a', '5', '0', 'F', 'A', '6', 'b', '1', 'B', '7', 'c', '2', 'C'):
                        states.append((206, self.reader.save()))
                        continue
                    break
                case 193:
                    if ch in ('5', '0', 'F', 'b', 'A', '6', '1', 'c', 'B', '7', '2', 'd', 'C', '8', 'E', '3', 'e', 'D', '9', '4', 'f', 'a'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 194:
                    if ch in ('0', 'A', 'b', '6', 'F', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5'):
                        states.append((207, self.reader.save()))
                        continue
                    break
                case 195:
                    if ch in ('3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8'):
                        states.append((208, self.reader.save()))
                        continue
                    break
                case 196:
                    if ch in ('A', '4', 'f', '6', 'B', '5', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 197:
                    if ch in ('D', '7', 'C', '9', 'E', '8', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B'):
                        states.append((209, self.reader.save()))
                        continue
                    break
                case 198:
                    if ch in ('6', 'f', 'B', '5', '7', 'A', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4'):
                        states.append((210, self.reader.save()))
                        continue
                    break
                case 199:
                    if ch in ('9', 'C', 'E', '8', 'a', 'D', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7'):
                        states.append((211, self.reader.save()))
                        continue
                    break
                case 200:
                    if ch in ('2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', '1', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', 'c', '7'):
                        states.append((212, self.reader.save()))
                        continue
                    break
                case 201:
                    if ch in ('d', 'l', 'f'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 202:
                    if ch in ('L', 'F', 'D'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 203:
                    if ch in ('0', '8', '3', '6', '1', '9', '4', '7', '2', '5'):
                        states.append((187, self.reader.save()))
                        continue
                    break
                case 204:
                    if ch in ('3', '6', '1', '9', '4', '8', '7', '2', '5', '0'):
                        states.append((188, self.reader.save()))
                        continue
                    break
                case 205:
                    if ch in ('b', '0', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', 'F', '5', 'A'):
                        states.append((213, self.reader.save()))
                        continue
                    break
                case 206:
                    if ch in ('0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'F', 'a', '5', 'A'):
                        states.append((214, self.reader.save()))
                        continue
                    break
                case 207:
                    if ch in ('4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 208:
                    if ch in ('2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '1', '0', 'A', 'b', '6', 'B', 'c', '7'):
                        states.append((215, self.reader.save()))
                        continue
                    break
                case 209:
                    if ch in ('B', '5', 'A', '7', 'C', '6', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f'):
                        states.append((216, self.reader.save()))
                        continue
                    break
                case 210:
                    if ch in ('3', '5', 'e', 'A', '4', '6', 'f', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 211:
                    if ch in ('6', '8', 'B', 'D', '7', '9', 'C', 'E', 'a', '0', 'F', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A'):
                        states.append((217, self.reader.save()))
                        continue
                    break
                case 212:
                    if ch in ('6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', '0', 'a', '5', 'F', 'A', 'b'):
                        states.append((218, self.reader.save()))
                        continue
                    break
                case 213:
                    if ch in ('a', '5', 'E', '0', 'A', 'F', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'f'):
                        states.append((219, self.reader.save()))
                        continue
                    break
                case 214:
                    if ch in ('f', 'a', 'E', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4'):
                        states.append((220, self.reader.save()))
                        continue
                    break
                case 215:
                    if ch in ('a', '0', '5', 'F', 'A', 'b', '1', '6', 'B', 'c', '2', '7', 'C', 'd', '3', '8', 'D', 'e', '4', '9', 'E', 'f'):
                        states.append((221, self.reader.save()))
                        continue
                    break
                case 216:
                    if ch in ('5', 'e', 'A', '4', '6', 'f', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3'):
                        states.append((222, self.reader.save()))
                        continue
                    break
                case 217:
                    if ch in ('A', '4', 'f', '6', 'B', '5', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e'):
                        states.append((223, self.reader.save()))
                        continue
                    break
                case 218:
                    if ch in ('5', '0', 'F', 'E', 'b', 'A', '6', '1', 'c', 'B', '7', '2', 'd', 'C', '8', '3', 'e', 'D', '9', '4', 'f', 'a'):
                        states.append((224, self.reader.save()))
                        continue
                    break
                case 219:
                    if ch in ('3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 220:
                    if ch in ('e', '9', '4', 'E', 'f', 'a', '5', 'F', 'D', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3'):
                        states.append((225, self.reader.save()))
                        continue
                    break
                case 221:
                    if ch in ('9', '4', 'E', 'f', 'a', '5', 'F', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', 'D', '8', '3', 'e'):
                        states.append((226, self.reader.save()))
                        continue
                    break
                case 222:
                    if ch in ('2', '4', 'd', 'f', '3', '5', 'e', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', '0', 'F', 'b', '1', 'c'):
                        states.append((227, self.reader.save()))
                        continue
                    break
                case 223:
                    if ch in ('f', '3', 'e', '5', 'A', '4', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd'):
                        states.append((228, self.reader.save()))
                        continue
                    break
                case 224:
                    if ch in ('e', '9', '4', 'E', 'f', 'a', '5', 'F', 'A', '0', 'D', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 225:
                    if ch in ('7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '0', '5', 'F', 'A', 'b', '1', '6', 'B', 'c'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 226:
                    if ch in ('d', '8', 'C', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2'):
                        states.append((229, self.reader.save()))
                        continue
                    break
                case 227:
                    if ch in ('d', '1', 'c', '3', 'e', '2', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 228:
                    if ch in ('3', 'c', 'e', '2', '4', 'd', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1'):
                        states.append((230, self.reader.save()))
                        continue
                    break
                case 229:
                    if ch in ('c', '7', '2', 'C', 'd', '8', '1', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 230:
                    if ch in ('0', '2', 'b', 'd', '1', '3', 'c', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case _:
                    break
        while states:
            state, back_index = states.pop()
            text = ''
            location = Location([])
            for i in range(start_index, back_index):
                text += self.reader.hasread[i][0]
                location += self.reader.hasread[i][1]
            match state:
                case 1:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 2:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 3:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 4:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 5:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 7:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 8:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 9:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 10:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 11:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 12:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 13:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 14:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 15:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 16:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 17:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 19:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 20:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 21:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 22:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 23:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 24:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 25:
                    self.reader.restore(back_index)
                    return Token(TokenKind.END, location, text)
                case 28:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 29:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 30:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 34:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 39:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 43:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 45:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 46:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 49:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 51:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 52:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 53:
                    self.reader.restore(back_index)
                    return Token(TokenKind.STRINGLITERAL, location, text)
                case 54:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 56:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 57:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 60:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 61:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 64:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 65:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 66:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 68:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 69:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 71:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 74:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 78:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 82:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 83:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 86:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 87:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 92:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 93:
                    self.reader.restore(back_index)
                    return Token(TokenKind.CHARCONST, location, text)
                case 94:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 98:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 105:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 106:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 107:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 111:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 117:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 118:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 121:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 123:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 136:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 138:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 140:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 146:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 147:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 150:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 151:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 152:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 153:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 156:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 160:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 161:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 162:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 177:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 179:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 187:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 188:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
        self.reader.restore(start_index)
        return None