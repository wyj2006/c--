from Lex.LexerBase import LexerBase
from copy import deepcopy
from Basic import *

def check_keyword(a):
    if a.text in Token.keywords:
        a.kind = Token.keywords[a.text]
    return a

def check_punctuator(a):
    return Token(Token.punctuator[a[1]], a[0], a[1])

class Gen_Lexer(LexerBase):

    def getNewToken(self):
        start_index = self.reader.save()
        states = [(0, start_index)]
        while True:
            ch, loc = self.reader.next()
            match states[-1][0]:
                case 0:
                    if ch == '-':
                        states.append((1, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((2, self.reader.save()))
                        continue
                    if ch == '*':
                        states.append((4, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    if ch == '0':
                        states.append((7, self.reader.save()))
                        continue
                    if ch == '%':
                        states.append((8, self.reader.save()))
                        continue
                    if ch == '|':
                        states.append((10, self.reader.save()))
                        continue
                    if ch == '+':
                        states.append((11, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((12, self.reader.save()))
                        continue
                    if ch == '[':
                        states.append((13, self.reader.save()))
                        continue
                    if ch == '!':
                        states.append((14, self.reader.save()))
                        continue
                    if ch == '?':
                        states.append((15, self.reader.save()))
                        continue
                    if ch == ']':
                        states.append((16, self.reader.save()))
                        continue
                    if ch == '^':
                        states.append((17, self.reader.save()))
                        continue
                    if ch == ':':
                        states.append((18, self.reader.save()))
                        continue
                    if ch == '~':
                        states.append((19, self.reader.save()))
                        continue
                    if ch == '&':
                        states.append((20, self.reader.save()))
                        continue
                    if ch == '(':
                        states.append((21, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((22, self.reader.save()))
                        continue
                    if ch == ';':
                        states.append((23, self.reader.save()))
                        continue
                    if ch == '<':
                        states.append((24, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((25, self.reader.save()))
                        continue
                    if ch == ')':
                        states.append((26, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((27, self.reader.save()))
                        continue
                    if ch == '/':
                        states.append((28, self.reader.save()))
                        continue
                    if ch == '{':
                        states.append((29, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == ',':
                        states.append((31, self.reader.save()))
                        continue
                    if ch == '}':
                        states.append((32, self.reader.save()))
                        continue
                    if ch == '#':
                        states.append((33, self.reader.save()))
                        continue
                    if ch == '':
                        states.append((34, self.reader.save()))
                        continue
                    if ch in ('5', '2', '6', '4', '3', '7', '8', '9', '1'):
                        states.append((3, self.reader.save()))
                        continue
                    if ch in ('e', 'r', 'E', 'R', 'f', 's', 'F', 'S', 'g', 't', 'G', 'T', 'h', 'H', 'i', 'v', 'I', 'V', 'j', 'w', 'J', 'W', 'k', 'x', 'K', 'X', 'l', 'y', '_', 'Y', 'm', 'z', 'M', 'a', 'Z', 'n', 'A', 'N', 'b', 'o', 'B', 'O', 'c', 'p', 'C', 'P', 'd', 'q', 'D', 'Q'):
                        states.append((5, self.reader.save()))
                        continue
                    if ch in ('U', 'L'):
                        states.append((9, self.reader.save()))
                        continue
                    if self.other_identifier_start(ch):
                        states.append((5, self.reader.save()))
                        continue
                    break
                case 1:
                    if ch == '-':
                        states.append((36, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((37, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((38, self.reader.save()))
                        continue
                    break
                case 2:
                    if ch == '8':
                        states.append((9, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((39, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((27, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((25, self.reader.save()))
                        continue
                    if ch in ('P', 'w', 'd', 'J', '2', '9', 'q', 'W', 'D', 'k', '3', 'Q', 'x', 'e', 'K', 'r', 'X', 'E', 'l', '4', 'R', 'y', 'f', 'L', 'j', 's', '_', 'Y', 'F', 'm', 'S', '5', 'z', 'g', 'M', 't', 'a', 'Z', 'G', 'n', '6', 'T', 'A', '0', 'h', 'N', 'u', 'b', 'H', 'o', '7', 'U', 'B', '1', 'i', 'O', 'v', 'c', 'I', 'p', 'V', 'C'):
                        states.append((5, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((5, self.reader.save()))
                        continue
                    break
                case 3:
                    if ch == "'":
                        states.append((40, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((41, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((42, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((43, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((45, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((46, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((48, self.reader.save()))
                        continue
                    if ch in ('9', '6', '8', '3', '5', '0', '2', '7', '4', '1'):
                        states.append((3, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((44, self.reader.save()))
                        continue
                    break
                case 4:
                    if ch == '=':
                        states.append((49, self.reader.save()))
                        continue
                    break
                case 5:
                    if ch == '\\':
                        states.append((39, self.reader.save()))
                        continue
                    if ch in ('P', 'w', 'd', 'J', '9', 'q', 'W', 'D', 'k', '3', 'Q', 'x', 'e', 'K', 'r', 'X', 'E', 'l', '4', 'R', 'y', 'f', 'L', 'j', 's', '_', 'Y', 'F', 'm', 'S', '5', 'z', 'g', 'M', 't', 'a', 'Z', 'G', 'n', '6', 'T', 'A', '0', 'h', 'N', 'u', 'b', 'H', 'o', '7', 'U', 'B', '1', 'i', 'O', 'v', 'c', 'I', 'p', '8', 'V', 'C', '2'):
                        states.append((5, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((5, self.reader.save()))
                        continue
                    break
                case 6:
                    if ch == '=':
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 7:
                    if ch == "'":
                        states.append((53, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((42, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((43, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((57, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((58, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((59, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((60, self.reader.save()))
                        continue
                    if ch in ('X', 'x'):
                        states.append((51, self.reader.save()))
                        continue
                    if ch in ('9', '8'):
                        states.append((52, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((54, self.reader.save()))
                        continue
                    if ch in ('5', '6', '2', '3', '0', '7', '4', '1'):
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('B', 'b'):
                        states.append((56, self.reader.save()))
                        continue
                    break
                case 8:
                    if ch == '=':
                        states.append((61, self.reader.save()))
                        continue
                    break
                case 9:
                    if ch == '\\':
                        states.append((39, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((27, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((25, self.reader.save()))
                        continue
                    if ch in ('P', 'w', 'd', 'J', '2', '9', 'q', 'W', 'D', 'k', '3', 'Q', 'x', 'e', 'K', 'r', 'X', 'E', 'l', '4', 'R', 'y', 'f', 'L', 'j', 's', '_', 'Y', 'F', 'm', 'S', '5', 'z', 'g', 'M', 't', 'a', 'Z', 'G', 'n', '6', 'T', 'A', '0', 'h', 'N', 'u', 'b', 'H', 'o', '7', 'U', 'B', '1', 'i', 'O', 'v', 'c', 'I', 'p', '8', 'V', 'C'):
                        states.append((5, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((5, self.reader.save()))
                        continue
                    break
                case 10:
                    if ch == '|':
                        states.append((62, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((63, self.reader.save()))
                        continue
                    break
                case 11:
                    if ch == '+':
                        states.append((64, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 12:
                    if ch == 'u':
                        states.append((66, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 13:
                    break
                case 14:
                    if ch == '=':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 15:
                    break
                case 16:
                    break
                case 17:
                    if ch == '=':
                        states.append((69, self.reader.save()))
                        continue
                    break
                case 18:
                    if ch == ':':
                        states.append((70, self.reader.save()))
                        continue
                    break
                case 19:
                    break
                case 20:
                    if ch == '=':
                        states.append((71, self.reader.save()))
                        continue
                    if ch == '&':
                        states.append((72, self.reader.save()))
                        continue
                    break
                case 21:
                    break
                case 22:
                    if ch == '.':
                        states.append((74, self.reader.save()))
                        continue
                    if ch in ('3', '6', '7', '0', '4', '1', '8', '9', '2', '5'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 23:
                    break
                case 24:
                    if ch == '<':
                        states.append((75, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((76, self.reader.save()))
                        continue
                    break
                case 25:
                    if ch == '\\':
                        states.append((77, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 26:
                    break
                case 27:
                    if ch == '\\':
                        states.append((79, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((81, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 28:
                    if ch == '=':
                        states.append((82, self.reader.save()))
                        continue
                    break
                case 29:
                    break
                case 30:
                    if ch == '=':
                        states.append((83, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((84, self.reader.save()))
                        continue
                    break
                case 31:
                    break
                case 32:
                    break
                case 33:
                    if ch == '#':
                        states.append((85, self.reader.save()))
                        continue
                    break
                case 34:
                    break
                case 35:
                    break
                case 36:
                    break
                case 37:
                    break
                case 38:
                    break
                case 39:
                    if ch == 'u':
                        states.append((86, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((87, self.reader.save()))
                        continue
                    break
                case 40:
                    if ch in ('9', '6', '8', '3', '5', '0', '2', '7', '4', '1'):
                        states.append((3, self.reader.save()))
                        continue
                    break
                case 41:
                    if ch == 'L':
                        states.append((89, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 42:
                    if ch == 'd':
                        states.append((90, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((94, self.reader.save()))
                        continue
                    if ch in ('F', 'l', 'L', 'f'):
                        states.append((91, self.reader.save()))
                        continue
                    if ch in ('7', '4', '1', '9', '3', '6', '0', '8', '2', '5'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 43:
                    if ch in ('1', '0', '9', '5', '4', '2', '6', '3', '7', '8'):
                        states.append((95, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((96, self.reader.save()))
                        continue
                    break
                case 44:
                    if ch == 'W':
                        states.append((97, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((99, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((100, self.reader.save()))
                        continue
                    break
                case 45:
                    if ch == 'B':
                        states.append((101, self.reader.save()))
                        continue
                    break
                case 46:
                    if ch == 'l':
                        states.append((89, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 47:
                    if ch in ('2', '6', '1', '3', '7', '0', '4', '8', '9', '5'):
                        states.append((102, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((103, self.reader.save()))
                        continue
                    break
                case 48:
                    if ch == 'b':
                        states.append((101, self.reader.save()))
                        continue
                    break
                case 49:
                    break
                case 50:
                    break
                case 51:
                    if ch == '.':
                        states.append((105, self.reader.save()))
                        continue
                    if ch in ('D', '1', '9', '0', '5', 'e', 'C', 'd', '3', 'A', 'c', '8', '7', 'F', 'E', '2', 'a', '6', 'f', '4', 'B', 'b'):
                        states.append((104, self.reader.save()))
                        continue
                    break
                case 52:
                    if ch == "'":
                        states.append((106, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((42, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((43, self.reader.save()))
                        continue
                    if ch in ('1', '9', '6', '3', '0', '8', '5', '2', '7', '4'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 53:
                    if ch in ('9', '8'):
                        states.append((52, self.reader.save()))
                        continue
                    if ch in ('6', '5', '3', '2', '0', '7', '4', '1'):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 54:
                    if ch == 'L':
                        states.append((107, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((108, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((109, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((110, self.reader.save()))
                        continue
                    break
                case 55:
                    if ch == "'":
                        states.append((53, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((42, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((43, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((57, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((58, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((59, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((60, self.reader.save()))
                        continue
                    if ch in ('9', '8'):
                        states.append((52, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((54, self.reader.save()))
                        continue
                    if ch in ('5', '6', '2', '3', '0', '7', '4', '1'):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 56:
                    if ch in ('1', '0'):
                        states.append((111, self.reader.save()))
                        continue
                    break
                case 57:
                    if ch == 'l':
                        states.append((112, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 58:
                    if ch == 'L':
                        states.append((112, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 59:
                    if ch == 'b':
                        states.append((113, self.reader.save()))
                        continue
                    break
                case 60:
                    if ch == 'B':
                        states.append((113, self.reader.save()))
                        continue
                    break
                case 61:
                    break
                case 62:
                    break
                case 63:
                    break
                case 64:
                    break
                case 65:
                    break
                case 66:
                    if ch in ('e', '6', '3', 'b', 'D', '1', '0', 'A', '8', 'd', 'F', '5', '2', 'a', 'B', 'C', 'f', '7', '4', 'c', 'E', '9'):
                        states.append((114, self.reader.save()))
                        continue
                    break
                case 67:
                    if ch in ('1', '9', 'B', '6', 'e', '3', 'b', 'D', '0', 'A', '8', '5', 'd', 'F', '2', 'a', 'C', '7', 'f', '4', 'c', 'E'):
                        states.append((115, self.reader.save()))
                        continue
                    break
                case 68:
                    break
                case 69:
                    break
                case 70:
                    break
                case 71:
                    break
                case 72:
                    break
                case 73:
                    if ch == 'd':
                        states.append((90, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((116, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((94, self.reader.save()))
                        continue
                    if ch in ('0', '8', '5', '2', '7', '4', '1', '9', '6', '3'):
                        states.append((73, self.reader.save()))
                        continue
                    if ch in ('F', 'l', 'L', 'f'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 74:
                    if ch == '.':
                        states.append((117, self.reader.save()))
                        continue
                    break
                case 75:
                    if ch == '=':
                        states.append((118, self.reader.save()))
                        continue
                    break
                case 76:
                    break
                case 77:
                    if ch == 'u':
                        states.append((119, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((121, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((122, self.reader.save()))
                        continue
                    if ch in ('?', "'", 't', 'n', 'b', '\\', '"', 'v', 'r', 'f', 'a'):
                        states.append((78, self.reader.save()))
                        continue
                    if ch in ('6', '3', '5', '0', '2', '7', '4', '1'):
                        states.append((120, self.reader.save()))
                        continue
                    break
                case 78:
                    if ch == '\\':
                        states.append((123, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((124, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 79:
                    if ch == 'u':
                        states.append((126, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((127, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((128, self.reader.save()))
                        continue
                    if ch in ('1', '2', '7', '5', '0', '4', '3', '6'):
                        states.append((125, self.reader.save()))
                        continue
                    if ch in ('v', 't', 'f', '?', 'b', '"', 'n', '\\', 'r', 'a', "'"):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 80:
                    if ch == '\\':
                        states.append((129, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((81, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 81:
                    break
                case 82:
                    break
                case 83:
                    break
                case 84:
                    if ch == '=':
                        states.append((130, self.reader.save()))
                        continue
                    break
                case 85:
                    break
                case 86:
                    if ch in ('0', '8', 'A', '5', 'd', 'F', '2', 'a', '3', 'C', 'f', '7', '4', 'c', 'E', '1', '9', 'B', '6', 'e', 'b', 'D'):
                        states.append((131, self.reader.save()))
                        continue
                    break
                case 87:
                    if ch in ('3', 'b', 'D', '0', 'A', '8', 'd', '5', 'F', 'a', '2', 'C', 'f', '7', '4', 'c', 'E', '1', '9', 'B', 'e', '6'):
                        states.append((132, self.reader.save()))
                        continue
                    break
                case 88:
                    break
                case 89:
                    if ch in ('u', 'U'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 90:
                    if ch in ('f', 'l', 'd'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 91:
                    break
                case 92:
                    if ch in ('D', 'L', 'F'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 93:
                    if ch in ('6', '3', '7', '4', '0', '8', '1', '9', '5', '2'):
                        states.append((133, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((134, self.reader.save()))
                        continue
                    break
                case 94:
                    if ch in ('5', '2', '6', '3', '7', '0', '4', '1', '8', '9'):
                        states.append((135, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((136, self.reader.save()))
                        continue
                    break
                case 95:
                    if ch == "'":
                        states.append((137, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((138, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((139, self.reader.save()))
                        continue
                    if ch in ('6', '3', '0', '8', '5', '2', '7', '4', '1', '9'):
                        states.append((95, self.reader.save()))
                        continue
                    if ch in ('l', 'L', 'f', 'F'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 96:
                    if ch in ('1', '0', '9', '5', '2', '6', '3', '7', '4', '8'):
                        states.append((95, self.reader.save()))
                        continue
                    break
                case 97:
                    if ch == 'B':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 98:
                    if ch == 'l':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 99:
                    if ch == 'b':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 100:
                    if ch == 'L':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 101:
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 102:
                    if ch == "'":
                        states.append((140, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((138, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((139, self.reader.save()))
                        continue
                    if ch in ('l', 'L', 'f', 'F'):
                        states.append((91, self.reader.save()))
                        continue
                    if ch in ('0', '8', '2', '5', '4', '7', '1', '9', '6', '3'):
                        states.append((102, self.reader.save()))
                        continue
                    break
                case 103:
                    if ch in ('2', '6', '3', '7', '0', '4', '9', '8', '1', '5'):
                        states.append((102, self.reader.save()))
                        continue
                    break
                case 104:
                    if ch == 'W':
                        states.append((141, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((142, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((143, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((145, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((146, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((147, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((148, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((149, self.reader.save()))
                        continue
                    if ch in ('C', 'e', 'd', '3', 'A', 'c', '8', '7', 'F', 'E', 'a', '2', '1', '6', '0', 'D', 'f', '4', 'B', '9', 'b', '5'):
                        states.append((104, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((144, self.reader.save()))
                        continue
                    break
                case 105:
                    if ch in ('A', '8', '5', 'd', 'F', '2', 'a', 'C', 'f', '4', '7', 'c', 'E', '1', '9', 'B', 'D', '6', 'e', '3', 'b', '0'):
                        states.append((150, self.reader.save()))
                        continue
                    break
                case 106:
                    if ch in ('6', '1', '9', '3', '0', '4', '8', '5', '2', '7'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 107:
                    if ch == 'L':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 108:
                    if ch == 'b':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 109:
                    if ch == 'l':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 110:
                    if ch == 'B':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 111:
                    if ch == 'w':
                        states.append((152, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((153, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((154, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((155, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((156, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((151, self.reader.save()))
                        continue
                    if ch in ('1', '0'):
                        states.append((111, self.reader.save()))
                        continue
                    break
                case 112:
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 113:
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 114:
                    if ch in ('1', '9', 'B', 'E', 'e', '6', '3', 'b', 'D', '0', 'A', '8', 'd', 'F', '5', '2', 'a', 'C', '4', 'f', '7', 'c'):
                        states.append((157, self.reader.save()))
                        continue
                    break
                case 115:
                    if ch in ('4', 'c', 'E', '1', '9', '7', 'B', '6', 'e', '3', 'b', 'D', '0', '8', 'A', '5', 'd', 'F', '2', 'a', 'C', 'f'):
                        states.append((158, self.reader.save()))
                        continue
                    break
                case 116:
                    if ch in ('0', '4', '8', '1', '9', '5', '2', '6', '3', '7'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 117:
                    break
                case 118:
                    break
                case 119:
                    if ch in ('6', 'e', '3', 'b', 'D', '0', '8', 'A', 'd', '5', 'F', 'a', '2', 'C', '7', 'f', '4', 'c', 'E', '1', '9', 'B'):
                        states.append((159, self.reader.save()))
                        continue
                    break
                case 120:
                    if ch == '\\':
                        states.append((123, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((124, self.reader.save()))
                        continue
                    if ch in ('2', '4', '7', '1', '3', '6', '0', '5'):
                        states.append((160, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 121:
                    if ch in ('e', '6', '3', 'b', 'B', 'D', '0', 'A', '8', 'd', '5', 'F', '2', 'a', 'C', 'f', '7', '4', 'c', 'E', '1', '9'):
                        states.append((161, self.reader.save()))
                        continue
                    break
                case 122:
                    if ch in ('5', 'd', 'F', '2', 'a', 'C', '7', 'f', '4', 'c', 'E', '8', '9', '1', 'B', '6', 'e', '3', 'b', 'D', '0', 'A'):
                        states.append((162, self.reader.save()))
                        continue
                    break
                case 123:
                    if ch == 'u':
                        states.append((164, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((165, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((166, self.reader.save()))
                        continue
                    if ch in ('5', '7', '2', '4', '1', '6', '3', '0'):
                        states.append((163, self.reader.save()))
                        continue
                    if ch in ('v', 'r', 'f', 'a', '?', "'", 't', 'n', 'b', '\\', '"'):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 124:
                    break
                case 125:
                    if ch == '\\':
                        states.append((129, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((81, self.reader.save()))
                        continue
                    if ch in ('6', '3', '5', '0', '2', '1', '7', '4'):
                        states.append((167, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 126:
                    if ch in ('2', 'A', '7', 'D', 'F', '4', 'E', '1', '8', 'B', 'e', '6', 'b', '0', '5', 'f', 'd', '3', 'C', 'a', 'c', '9'):
                        states.append((168, self.reader.save()))
                        continue
                    break
                case 127:
                    if ch in ('6', 'd', 'a', '0', '5', 'c', 'D', '8', '4', '7', 'E', '1', 'B', '9', 'e', 'b', 'F', 'A', 'C', '2', 'f', '3'):
                        states.append((169, self.reader.save()))
                        continue
                    break
                case 128:
                    if ch in ('6', 'E', '3', 'D', 'b', '1', '0', 'A', 'd', 'a', 'F', '7', '4', 'e', 'B', '2', '5', 'c', '8', 'C', '9', 'f'):
                        states.append((170, self.reader.save()))
                        continue
                    break
                case 129:
                    if ch == 'x':
                        states.append((172, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((173, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((174, self.reader.save()))
                        continue
                    if ch in ('2', '4', '3', '5', '6', '0', '7', '1'):
                        states.append((171, self.reader.save()))
                        continue
                    if ch in ('\\', 'f', '?', 'r', 'a', 'n', "'", 't', 'b', '"', 'v'):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 130:
                    break
                case 131:
                    if ch in ('3', 'b', 'D', '0', '8', 'A', 'd', '5', 'F', 'a', '2', 'C', '7', 'f', '4', 'c', 'E', '1', '9', 'B', 'e', '6'):
                        states.append((175, self.reader.save()))
                        continue
                    break
                case 132:
                    if ch in ('e', '6', '3', 'b', 'D', '0', 'A', '8', 'd', 'F', '5', '2', 'a', '1', 'C', 'f', '7', '4', 'c', 'E', 'B', '9'):
                        states.append((176, self.reader.save()))
                        continue
                    break
                case 133:
                    if ch == 'd':
                        states.append((90, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((177, self.reader.save()))
                        continue
                    if ch in ('0', '8', '2', '5', '4', '7', '1', '9', '3', '6'):
                        states.append((133, self.reader.save()))
                        continue
                    if ch in ('F', 'l', 'L', 'f'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 134:
                    if ch in ('6', '3', '7', '4', '0', '8', '1', '9', '5', '2'):
                        states.append((133, self.reader.save()))
                        continue
                    break
                case 135:
                    if ch == "'":
                        states.append((178, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((90, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((92, self.reader.save()))
                        continue
                    if ch in ('3', '0', '8', '5', '2', '7', '4', '1', '9', '6'):
                        states.append((135, self.reader.save()))
                        continue
                    if ch in ('F', 'l', 'L', 'f'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 136:
                    if ch in ('9', '5', '2', '6', '3', '7', '0', '4', '8', '1'):
                        states.append((135, self.reader.save()))
                        continue
                    break
                case 137:
                    if ch in ('6', '3', '7', '0', '4', '8', '1', '9', '5', '2'):
                        states.append((95, self.reader.save()))
                        continue
                    break
                case 138:
                    if ch in ('L', 'F', 'D'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 139:
                    if ch in ('f', 'l', 'd'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 140:
                    if ch in ('0', '4', '7', '1', '8', '3', '9', '2', '5', '6'):
                        states.append((102, self.reader.save()))
                        continue
                    break
                case 141:
                    if ch == 'B':
                        states.append((179, self.reader.save()))
                        continue
                    break
                case 142:
                    if ch in ('e', 'C', '3', 'd', 'A', 'c', '8', 'F', '7', 'a', '2', 'E', '1', '6', 'f', 'D', '4', '0', 'B', '9', 'b', '5'):
                        states.append((104, self.reader.save()))
                        continue
                    break
                case 143:
                    if ch == 'P':
                        states.append((180, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((181, self.reader.save()))
                        continue
                    if ch in ('A', '8', '5', 'd', 'F', '2', 'a', 'C', 'f', '4', '7', 'c', 'E', 'b', '1', '9', 'B', '6', 'e', '3', 'D', '0'):
                        states.append((150, self.reader.save()))
                        continue
                    break
                case 144:
                    if ch == 'L':
                        states.append((182, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((183, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((184, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((185, self.reader.save()))
                        continue
                    break
                case 145:
                    if ch == 'l':
                        states.append((186, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 146:
                    if ch in ('5', '2', '6', '3', '7', '0', '4', '1', '8', '9'):
                        states.append((187, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((188, self.reader.save()))
                        continue
                    break
                case 147:
                    if ch == 'L':
                        states.append((186, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 148:
                    if ch == 'b':
                        states.append((179, self.reader.save()))
                        continue
                    break
                case 149:
                    if ch in ('1', '8', '9', '5', '2', '6', '4', '3', '7', '0'):
                        states.append((189, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((190, self.reader.save()))
                        continue
                    break
                case 150:
                    if ch == 'P':
                        states.append((180, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((181, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((191, self.reader.save()))
                        continue
                    if ch in ('5', 'd', 'F', '2', 'a', 'C', 'f', '4', '7', 'c', 'E', '1', '9', 'B', '6', 'e', '3', 'b', 'D', '0', 'A', '8'):
                        states.append((150, self.reader.save()))
                        continue
                    break
                case 151:
                    if ch == 'L':
                        states.append((192, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((193, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((194, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((195, self.reader.save()))
                        continue
                    break
                case 152:
                    if ch == 'b':
                        states.append((196, self.reader.save()))
                        continue
                    break
                case 153:
                    if ch == 'L':
                        states.append((197, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 154:
                    if ch in ('1', '0'):
                        states.append((111, self.reader.save()))
                        continue
                    break
                case 155:
                    if ch == 'l':
                        states.append((197, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 156:
                    if ch == 'B':
                        states.append((196, self.reader.save()))
                        continue
                    break
                case 157:
                    if ch in ('4', '7', 'c', 'E', '1', '9', 'B', 'e', '3', '6', 'b', 'D', '0', 'A', '8', '5', 'd', 'F', '2', 'a', 'C', 'f'):
                        states.append((198, self.reader.save()))
                        continue
                    break
                case 158:
                    if ch in ('7', 'f', '4', 'c', '2', 'E', '1', '9', 'B', 'e', '6', '3', 'b', 'D', '0', '8', 'A', 'd', '5', 'F', 'a', 'C'):
                        states.append((199, self.reader.save()))
                        continue
                    break
                case 159:
                    if ch in ('B', '6', 'e', '3', 'b', 'D', '0', 'A', '8', 'd', '5', '1', 'F', '2', 'a', 'C', '7', 'f', '4', 'c', 'E', '9'):
                        states.append((200, self.reader.save()))
                        continue
                    break
                case 160:
                    if ch == '\\':
                        states.append((123, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((124, self.reader.save()))
                        continue
                    if ch in ('6', '3', '0', '5', '2', '7', '4', '1'):
                        states.append((78, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 161:
                    if ch in ('4', '1', '9', 'B', 'e', 'E', '6', '3', 'b', 'D', 'A', '0', '8', 'd', 'F', '5', '2', 'a', 'C', 'f', '7', 'c'):
                        states.append((201, self.reader.save()))
                        continue
                    break
                case 162:
                    if ch == '\\':
                        states.append((123, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((124, self.reader.save()))
                        continue
                    if ch in ('C', 'f', 'c', '9', '6', '3', 'D', '0', 'A', '5', 'd', 'a', '7', '4', 'E', '1', 'B', 'e', 'b', '8', 'F', '2'):
                        states.append((162, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 163:
                    if ch == '\\':
                        states.append((123, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((124, self.reader.save()))
                        continue
                    if ch in ('3', '6', '0', '5', '2', '7', '4', '1'):
                        states.append((202, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 164:
                    if ch in ('5', 'F', '2', 'a', 'C', '7', 'f', '4', 'c', '8', 'E', '9', '1', 'B', '6', 'e', '3', 'b', 'D', '0', 'A', 'd'):
                        states.append((203, self.reader.save()))
                        continue
                    break
                case 165:
                    if ch in ('8', 'd', 'F', '5', '2', 'a', 'C', 'f', '7', 'c', '4', 'E', '1', '9', 'B', 'e', '6', '3', 'b', '0', 'D', 'A'):
                        states.append((204, self.reader.save()))
                        continue
                    break
                case 166:
                    if ch in ('1', '9', 'B', '6', 'e', '3', 'b', '4', 'D', '0', 'A', '8', '5', 'd', 'F', '2', 'a', 'C', '7', 'f', 'c', 'E'):
                        states.append((205, self.reader.save()))
                        continue
                    break
                case 167:
                    if ch == '\\':
                        states.append((129, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((81, self.reader.save()))
                        continue
                    if ch in ('7', '4', '1', '2', '5', '0', '3', '6'):
                        states.append((80, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 168:
                    if ch in ('a', 'C', '2', '4', '9', 'b', '1', '7', 'E', 'A', 'd', 'B', 'e', 'F', '5', '8', 'c', '3', 'f', 'D', '0', '6'):
                        states.append((206, self.reader.save()))
                        continue
                    break
                case 169:
                    if ch in ('8', 'd', 'F', 'a', '1', 'C', '7', '4', 'f', 'E', '0', 'c', 'B', '9', 'e', '6', 'b', 'D', 'A', '5', '2', '3'):
                        states.append((207, self.reader.save()))
                        continue
                    break
                case 170:
                    if ch == '\\':
                        states.append((129, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((81, self.reader.save()))
                        continue
                    if ch in ('f', 'a', 'b', '5', 'C', 'c', '6', 'F', '1', 'D', 'A', '0', 'B', '7', 'd', '2', 'E', '8', 'e', '3', '9', '4'):
                        states.append((170, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 171:
                    if ch == '\\':
                        states.append((129, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((81, self.reader.save()))
                        continue
                    if ch in ('2', '5', '7', '4', '1', '0', '6', '3'):
                        states.append((208, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 172:
                    if ch in ('F', 'A', '5', '2', 'C', 'd', 'f', 'a', 'c', '7', '9', '4', 'E', '1', 'B', '6', '0', '3', 'D', 'e', 'b', '8'):
                        states.append((209, self.reader.save()))
                        continue
                    break
                case 173:
                    if ch in ('1', 'B', '6', '3', 'D', 'e', '0', 'A', 'b', 'd', '8', '7', 'a', 'F', '2', '5', '9', 'C', '4', 'E', 'f', 'c'):
                        states.append((210, self.reader.save()))
                        continue
                    break
                case 174:
                    if ch in ('4', 'E', '6', '1', '3', 'B', 'D', '0', 'e', 'A', 'b', 'd', '8', 'a', '5', 'F', '7', '2', 'C', 'f', 'c', '9'):
                        states.append((211, self.reader.save()))
                        continue
                    break
                case 175:
                    if ch in ('e', '6', '3', 'b', 'D', '0', 'A', '8', 'd', '5', 'F', '2', 'a', 'C', 'f', '1', '7', '4', 'c', '9', 'E', 'B'):
                        states.append((212, self.reader.save()))
                        continue
                    break
                case 176:
                    if ch in ('9', 'B', 'e', '6', '3', 'b', 'D', '0', 'A', '8', 'd', '4', 'F', '5', '2', 'a', 'C', 'f', '7', 'E', 'c', '1'):
                        states.append((213, self.reader.save()))
                        continue
                    break
                case 177:
                    if ch in ('0', '4', '7', '1', '8', '9', '2', '5', '3', '6'):
                        states.append((133, self.reader.save()))
                        continue
                    break
                case 178:
                    if ch in ('3', '7', '4', '0', '8', '1', '9', '5', '2', '6'):
                        states.append((135, self.reader.save()))
                        continue
                    break
                case 179:
                    if ch in ('u', 'U'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 180:
                    if ch in ('8', '1', '9', '5', '2', '6', '0', '3', '4', '7'):
                        states.append((214, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((215, self.reader.save()))
                        continue
                    break
                case 181:
                    if ch in ('0', '7', '4', '1', '8', '3', '9', '2', '5', '6'):
                        states.append((216, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((217, self.reader.save()))
                        continue
                    break
                case 182:
                    if ch == 'L':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 183:
                    if ch == 'b':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 184:
                    if ch == 'B':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 185:
                    if ch == 'l':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 186:
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 187:
                    if ch == "'":
                        states.append((218, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((219, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((220, self.reader.save()))
                        continue
                    if ch in ('3', '0', '8', '5', '2', '7', '4', '1', '9', '6'):
                        states.append((187, self.reader.save()))
                        continue
                    if ch in ('l', 'L', 'f', 'F'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 188:
                    if ch in ('9', '5', '2', '6', '3', '7', '0', '4', '8', '1'):
                        states.append((187, self.reader.save()))
                        continue
                    break
                case 189:
                    if ch == "'":
                        states.append((221, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((219, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((220, self.reader.save()))
                        continue
                    if ch in ('6', '3', '0', '8', '5', '2', '7', '4', '9', '1'):
                        states.append((189, self.reader.save()))
                        continue
                    if ch in ('l', 'L', 'f', 'F'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 190:
                    if ch in ('1', '8', '9', '5', '2', '6', '3', '7', '0', '4'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 191:
                    if ch in ('5', 'd', 'F', '2', 'a', 'C', 'f', '4', '7', 'c', 'E', '1', '9', 'B', '6', 'e', '3', 'b', 'D', '0', 'A', '8'):
                        states.append((150, self.reader.save()))
                        continue
                    break
                case 192:
                    if ch == 'L':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 193:
                    if ch == 'l':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 194:
                    if ch == 'B':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 195:
                    if ch == 'b':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 196:
                    if ch in ('u', 'U'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 197:
                    if ch in ('U', 'u'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 198:
                    if ch in ('7', 'f', '4', 'c', 'E', '1', '9', 'B', 'e', '6', 'C', '3', 'b', 'D', '0', '8', 'A', '5', 'd', 'F', '2', 'a'):
                        states.append((5, self.reader.save()))
                        continue
                    break
                case 199:
                    if ch in ('2', 'a', 'C', 'f', '7', '4', 'c', 'E', '9', '1', 'B', 'e', '6', '3', 'b', 'D', '0', 'A', '8', 'F', 'd', '5'):
                        states.append((222, self.reader.save()))
                        continue
                    break
                case 200:
                    if ch in ('E', '1', '9', 'B', 'e', '6', '3', 'b', 'D', 'A', '0', '8', '4', 'd', '5', 'F', '2', 'a', 'C', 'f', '7', 'c'):
                        states.append((223, self.reader.save()))
                        continue
                    break
                case 201:
                    if ch in ('7', '4', 'c', 'E', '1', '9', 'B', 'e', '6', '3', 'b', 'D', '0', 'A', '8', 'd', 'F', '5', '2', 'a', 'C', 'f'):
                        states.append((224, self.reader.save()))
                        continue
                    break
                case 202:
                    if ch == '\\':
                        states.append((123, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((124, self.reader.save()))
                        continue
                    if ch in ('2', '7', '4', '5', '1', '6', '3', '0'):
                        states.append((78, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 203:
                    if ch in ('8', 'd', '5', 'F', '2', 'a', 'C', 'f', '7', 'c', '0', '4', 'E', '1', '9', 'B', 'e', '6', '3', 'b', 'D', 'A'):
                        states.append((225, self.reader.save()))
                        continue
                    break
                case 204:
                    if ch in ('0', 'A', '8', 'd', 'F', '5', '2', 'a', 'C', 'f', 'b', '7', '4', 'c', 'E', 'D', '1', '9', 'B', 'e', '6', '3'):
                        states.append((226, self.reader.save()))
                        continue
                    break
                case 205:
                    if ch == '\\':
                        states.append((123, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((124, self.reader.save()))
                        continue
                    if ch in ('B', 'e', 'b', '8', '5', 'F', '2', 'C', 'f', 'c', '9', '6', '3', '0', 'D', 'A', 'd', 'a', '7', '4', 'E', '1'):
                        states.append((205, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 206:
                    if ch in ('A', 'D', 'd', '9', 'c', '0', 'a', '2', 'B', '7', '4', 'E', '3', '1', 'f', 'e', '6', '5', 'b', '8', 'F', 'C'):
                        states.append((227, self.reader.save()))
                        continue
                    break
                case 207:
                    if ch in ('b', '0', 'A', '8', 'd', 'a', '5', 'F', '2', 'C', '7', 'f', '4', 'E', 'c', '1', 'B', '9', '3', 'e', '6', 'D'):
                        states.append((228, self.reader.save()))
                        continue
                    break
                case 208:
                    if ch == '\\':
                        states.append((129, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((81, self.reader.save()))
                        continue
                    if ch in ('6', '3', '0', '5', '2', '7', '4', '1'):
                        states.append((80, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 209:
                    if ch == '\\':
                        states.append((129, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((81, self.reader.save()))
                        continue
                    if ch in ('5', '7', 'C', 'E', '1', 'c', 'e', '6', '8', '0', 'D', 'F', '2', 'd', 'f', '9', '3', 'A', 'a', '4', 'B', 'b'):
                        states.append((209, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 210:
                    if ch in ('4', '9', 'E', '1', 'B', '6', '3', 'D', 'e', '0', 'A', 'b', 'd', '8', 'a', '5', 'F', '2', 'C', '7', 'f', 'c'):
                        states.append((229, self.reader.save()))
                        continue
                    break
                case 211:
                    if ch in ('7', '9', '4', 'E', '1', '6', 'B', '3', 'e', 'D', '0', 'b', 'A', 'f', 'd', '8', 'a', '5', 'F', '2', 'C', 'c'):
                        states.append((230, self.reader.save()))
                        continue
                    break
                case 212:
                    if ch in ('1', '9', 'B', 'e', '6', '3', 'b', 'c', 'D', 'A', '0', '8', 'E', 'd', '5', 'F', '2', 'a', 'C', 'f', '7', '4'):
                        states.append((5, self.reader.save()))
                        continue
                    break
                case 213:
                    if ch in ('c', 'E', '1', '9', 'B', 'e', '4', '3', '6', 'b', 'D', '0', 'A', '8', 'd', 'F', '5', '2', 'a', 'C', 'f', '7'):
                        states.append((231, self.reader.save()))
                        continue
                    break
                case 214:
                    if ch == "'":
                        states.append((232, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((233, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((234, self.reader.save()))
                        continue
                    if ch in ('6', '3', '0', '8', '9', '5', '2', '7', '4', '1'):
                        states.append((214, self.reader.save()))
                        continue
                    if ch in ('l', 'L', 'f', 'F'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 215:
                    if ch in ('8', '1', '9', '5', '2', '6', '0', '3', '7', '4'):
                        states.append((214, self.reader.save()))
                        continue
                    break
                case 216:
                    if ch == "'":
                        states.append((235, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((233, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((234, self.reader.save()))
                        continue
                    if ch in ('1', '9', '6', '3', '0', '8', '5', '2', '7', '4'):
                        states.append((216, self.reader.save()))
                        continue
                    if ch in ('l', 'L', 'f', 'F'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 217:
                    if ch in ('0', '7', '4', '1', '8', '9', '2', '5', '6', '3'):
                        states.append((216, self.reader.save()))
                        continue
                    break
                case 218:
                    if ch in ('3', '7', '4', '0', '8', '1', '9', '5', '2', '6'):
                        states.append((187, self.reader.save()))
                        continue
                    break
                case 219:
                    if ch in ('F', 'D', 'L'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 220:
                    if ch in ('d', 'l', 'f'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 221:
                    if ch in ('2', '6', '3', '7', '0', '4', '8', '9', '1', '5'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 222:
                    if ch in ('7', '4', 'c', 'E', '1', '9', 'B', 'e', '6', 'b', '3', 'D', '0', 'A', '8', 'd', '5', 'F', '2', 'a', 'C', 'f'):
                        states.append((236, self.reader.save()))
                        continue
                    break
                case 223:
                    if ch in ('7', '4', 'c', 'E', '1', '9', 'B', 'e', '6', 'b', '3', 'D', '0', 'A', '8', 'd', '5', 'F', '2', 'a', 'C', 'f'):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 224:
                    if ch in ('f', '7', '4', 'c', 'E', '1', '9', 'B', 'e', '3', '6', 'C', 'b', 'D', '0', 'A', '8', 'd', 'F', '2', 'a', '5'):
                        states.append((237, self.reader.save()))
                        continue
                    break
                case 225:
                    if ch in ('0', 'A', '8', 'd', '5', 'D', 'F', '2', 'a', 'C', 'f', '7', '4', 'c', 'E', '1', '9', 'B', 'e', '3', '6', 'b'):
                        states.append((238, self.reader.save()))
                        continue
                    break
                case 226:
                    if ch in ('6', '3', 'b', 'D', '0', 'A', '8', 'd', 'F', '5', '2', 'a', 'C', 'f', '7', '4', 'c', 'E', '1', '9', 'B', 'e'):
                        states.append((239, self.reader.save()))
                        continue
                    break
                case 227:
                    if ch in ('0', 'A', '1', 'a', 'd', '6', 'C', '4', '7', 'f', 'E', 'B', 'e', '2', 'b', '9', 'F', '8', '5', '3', 'D', 'c'):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 228:
                    if ch in ('e', '3', 'D', 'b', '0', 'A', 'd', '8', 'a', '5', 'F', '2', 'C', '7', 'f', '4', 'E', 'c', '1', 'B', '9', '6'):
                        states.append((240, self.reader.save()))
                        continue
                    break
                case 229:
                    if ch in ('c', '7', '4', '9', 'E', '1', 'B', '3', '6', 'D', 'e', '0', 'A', 'b', 'd', 'f', '8', 'a', '5', 'F', '2', 'C'):
                        states.append((241, self.reader.save()))
                        continue
                    break
                case 230:
                    if ch in ('a', 'C', 'c', '7', '9', '4', 'E', '1', 'B', '6', '3', 'e', 'D', '0', 'b', 'A', 'd', '8', 'F', '5', '2', 'f'):
                        states.append((242, self.reader.save()))
                        continue
                    break
                case 231:
                    if ch in ('e', '3', '6', 'b', 'D', '0', 'A', '8', 'd', '5', 'F', '2', 'a', 'C', 'f', '4', '7', 'c', 'E', '1', '9', 'B'):
                        states.append((243, self.reader.save()))
                        continue
                    break
                case 232:
                    if ch in ('2', '6', '3', '7', '0', '4', '8', '1', '9', '5'):
                        states.append((214, self.reader.save()))
                        continue
                    break
                case 233:
                    if ch in ('l', 'd', 'f'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 234:
                    if ch in ('F', 'D', 'L'):
                        states.append((91, self.reader.save()))
                        continue
                    break
                case 235:
                    if ch in ('1', '9', '5', '2', '6', '4', '3', '7', '0', '8'):
                        states.append((216, self.reader.save()))
                        continue
                    break
                case 236:
                    if ch in ('f', '7', '4', 'c', 'E', '1', '9', 'B', '2', 'e', '6', '3', 'b', 'D', 'C', '0', 'A', '8', 'd', 'F', '5', 'a'):
                        states.append((244, self.reader.save()))
                        continue
                    break
                case 237:
                    if ch in ('c', 'E', '1', '9', 'B', 'e', '3', '6', 'b', 'D', '0', 'A', '8', '5', 'd', 'F', '2', 'a', 'C', '7', 'f', '4'):
                        states.append((245, self.reader.save()))
                        continue
                    break
                case 238:
                    if ch in ('6', '3', 'b', 'D', '0', 'A', '8', 'd', 'F', '5', 'a', '2', 'C', 'f', '7', '4', 'c', 'E', '1', '9', 'B', 'e'):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 239:
                    if ch in ('e', '3', '6', 'b', 'D', '0', 'A', '8', 'd', '5', 'F', '2', 'a', 'C', 'f', '4', '7', 'c', 'E', '1', '9', 'B'):
                        states.append((246, self.reader.save()))
                        continue
                    break
                case 240:
                    if ch in ('8', '0', 'A', '5', 'F', 'd', '2', 'C', 'a', 'f', 'c', '7', '4', 'E', '9', '1', 'B', '6', 'e', '3', 'D', 'b'):
                        states.append((247, self.reader.save()))
                        continue
                    break
                case 241:
                    if ch in ('f', 'a', 'c', '7', '9', '4', 'E', '6', '1', '3', 'B', 'd', 'D', '0', 'e', 'A', 'b', '8', 'F', '5', '2', 'C'):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 242:
                    if ch in ('d', '0', 'f', 'a', 'c', '5', '7', '9', '4', 'E', '1', 'B', '6', '3', 'e', 'D', 'A', 'b', 'C', '8', 'F', '2'):
                        states.append((248, self.reader.save()))
                        continue
                    break
                case 243:
                    if ch in ('1', '9', 'B', '6', 'e', '3', 'b', 'D', '0', 'A', '8', 'E', '5', 'd', 'F', '2', 'a', 'C', '7', 'f', '4', 'c'):
                        states.append((249, self.reader.save()))
                        continue
                    break
                case 244:
                    if ch in ('5', '2', 'a', 'C', 'F', 'f', '7', '4', 'c', 'E', 'B', '9', '1', 'e', '6', '3', 'b', 'D', '0', 'A', '8', 'd'):
                        states.append((250, self.reader.save()))
                        continue
                    break
                case 245:
                    if ch in ('7', 'f', '4', 'c', 'E', '1', '9', 'B', 'e', '6', '3', 'b', 'D', '0', '8', 'A', 'C', '5', 'd', 'F', '2', 'a'):
                        states.append((251, self.reader.save()))
                        continue
                    break
                case 246:
                    if ch in ('b', 'D', '0', 'A', '8', '5', 'd', 'F', '2', 'a', 'C', '7', 'f', '4', 'c', 'E', '1', '9', 'B', 'e', '6', '3'):
                        states.append((252, self.reader.save()))
                        continue
                    break
                case 247:
                    if ch in ('6', '3', 'D', '8', '0', 'A', '5', 'F', '2', 'C', 'd', 'f', 'a', 'c', '7', '9', '4', 'E', '1', 'B', 'e', 'b'):
                        states.append((253, self.reader.save()))
                        continue
                    break
                case 248:
                    if ch in ('4', 'f', 'E', '1', 'B', 'c', 'e', '9', 'b', '6', '3', 'D', '8', '0', 'A', '5', 'F', '2', 'C', 'd', 'a', '7'):
                        states.append((254, self.reader.save()))
                        continue
                    break
                case 249:
                    if ch in ('4', 'c', 'E', '1', '9', 'B', '6', 'e', '3', 'b', 'D', '0', '8', 'A', '7', '5', 'd', 'F', '2', 'a', 'C', 'f'):
                        states.append((255, self.reader.save()))
                        continue
                    break
                case 250:
                    if ch in ('8', 'd', 'F', '5', '2', 'a', 'C', 'f', '7', 'c', '4', 'E', '1', '9', 'B', 'e', '3', '6', 'b', 'D', '0', 'A'):
                        states.append((5, self.reader.save()))
                        continue
                    break
                case 251:
                    if ch in ('C', '2', '7', 'f', 'F', '4', 'c', 'E', '1', '9', 'B', '6', 'e', '3', 'b', 'D', '0', '8', 'A', '5', 'a', 'd'):
                        states.append((256, self.reader.save()))
                        continue
                    break
                case 252:
                    if ch in ('6', 'e', '3', 'b', 'D', '0', '8', 'A', 'd', '5', 'F', 'a', '2', 'C', '7', 'f', '4', 'c', 'E', '1', '9', 'B'):
                        states.append((257, self.reader.save()))
                        continue
                    break
                case 253:
                    if ch in ('b', '6', '3', 'D', '0', '8', 'A', 'F', '5', 'd', '2', 'C', 'a', 'f', 'e', 'c', '7', '4', 'E', '9', '1', 'B'):
                        states.append((258, self.reader.save()))
                        continue
                    break
                case 254:
                    if ch in ('5', '7', '2', 'C', '4', 'E', 'f', '1', 'B', 'c', 'e', '9', 'b', '6', '8', '3', 'D', 'F', '0', 'A', 'd', 'a'):
                        states.append((259, self.reader.save()))
                        continue
                    break
                case 255:
                    if ch in ('7', 'f', '4', 'c', 'E', '1', '9', 'B', 'e', '6', '3', 'b', 'D', '0', '8', 'A', '2', 'd', '5', 'F', 'a', 'C'):
                        states.append((5, self.reader.save()))
                        continue
                    break
                case 256:
                    if ch in ('5', 'F', '2', 'a', 'C', '7', 'f', '4', 'c', 'E', '9', '1', 'B', 'e', '6', '3', 'b', 'D', '0', '8', 'd', 'A'):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 257:
                    if ch in ('B', '6', 'e', '3', 'b', 'D', '0', 'A', '8', 'd', '5', '1', 'F', '2', 'a', 'C', '7', 'f', '4', 'c', 'E', '9'):
                        states.append((260, self.reader.save()))
                        continue
                    break
                case 258:
                    if ch in ('e', '9', 'b', '3', '6', 'D', '8', '0', 'F', 'A', '5', '2', 'C', 'd', 'f', 'a', 'c', '4', '7', 'E', '1', 'B'):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 259:
                    if ch in ('8', 'a', 'F', '2', '5', 'C', '7', '4', 'E', 'f', 'B', '1', 'c', 'e', '9', 'b', '3', '6', 'D', '0', 'A', 'd'):
                        states.append((261, self.reader.save()))
                        continue
                    break
                case 260:
                    if ch in ('E', '1', '9', 'B', 'e', '6', '3', 'b', 'D', 'A', '0', '8', '4', 'd', '5', 'F', '2', 'a', 'C', 'f', '7', 'c'):
                        states.append((78, self.reader.save()))
                        continue
                    break
                case 261:
                    if ch in ('d', '8', 'a', '5', 'F', '2', 'C', '7', '4', 'E', 'f', '1', 'B', 'c', 'e', '9', 'b', '6', '3', 'D', '0', 'A'):
                        states.append((80, self.reader.save()))
                        continue
                    break
                case _:
                    break
        while states:
            state, back_index = states.pop()
            text = ''
            location = None
            for i in range(start_index, back_index):
                text += self.reader.hasread[i][0]
                if location == None:
                    location = deepcopy(self.reader.hasread[i][1])
                else:
                    location += self.reader.hasread[i][1]
            match state:
                case 1:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 2:
                    self.reader.restore(back_index)
                    return check_keyword(Token(TokenKind.IDENTIFIER, location, text))
                case 3:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 4:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 5:
                    self.reader.restore(back_index)
                    return check_keyword(Token(TokenKind.IDENTIFIER, location, text))
                case 6:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 7:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 8:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 9:
                    self.reader.restore(back_index)
                    return check_keyword(Token(TokenKind.IDENTIFIER, location, text))
                case 10:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 11:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 13:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 14:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 15:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 16:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 17:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 18:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 19:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 20:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 21:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 22:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 23:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 24:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 26:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 27:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 28:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 29:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 30:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 31:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 32:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 33:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 34:
                    self.reader.restore(back_index)
                    return Token(TokenKind.END, location, text)
                case 36:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 37:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 38:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 41:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 42:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 44:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 46:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 49:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 50:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 54:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 55:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 57:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 58:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 61:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 62:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 63:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 64:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 65:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 68:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 69:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 70:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 71:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 72:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 73:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 75:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 76:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 77:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 79:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 80:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 81:
                    self.reader.restore(back_index)
                    return Token(TokenKind.STRINGLITERAL, location, text)
                case 82:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 83:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 84:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 85:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 88:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 89:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 91:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 95:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 98:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 100:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 101:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 102:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 104:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 107:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 109:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 111:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 112:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 113:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 117:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 118:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 123:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 124:
                    self.reader.restore(back_index)
                    return Token(TokenKind.CHARCONST, location, text)
                case 125:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 129:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 130:
                    self.reader.restore(back_index)
                    return check_punctuator((location, text))
                case 133:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 135:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 144:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 145:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 147:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 151:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 153:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 155:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 167:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 170:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 171:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 179:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 182:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 185:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 186:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 187:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 189:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 192:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 193:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 196:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 197:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 208:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 209:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 214:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 216:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
        self.reader.restore(start_index)
        return None