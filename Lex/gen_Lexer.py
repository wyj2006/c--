from Lex import LexerBase
from copy import deepcopy
from Basic import *

class Gen_Lexer(LexerBase):

    def getNewToken(self):
        start_index = self.reader.save()
        states = [(0, start_index)]
        while True:
            ch, loc = self.reader.next()
            match states[-1][0]:
                case 0:
                    if ch == '<':
                        states.append((2, self.reader.save()))
                        continue
                    if ch == '*':
                        states.append((3, self.reader.save()))
                        continue
                    if ch == '#':
                        states.append((5, self.reader.save()))
                        continue
                    if ch == '-':
                        states.append((6, self.reader.save()))
                        continue
                    if ch == '0':
                        states.append((7, self.reader.save()))
                        continue
                    if ch == '+':
                        states.append((8, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((9, self.reader.save()))
                        continue
                    if ch == '&':
                        states.append((11, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((12, self.reader.save()))
                        continue
                    if ch == '':
                        states.append((13, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((14, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((15, self.reader.save()))
                        continue
                    if ch == '^':
                        states.append((17, self.reader.save()))
                        continue
                    if ch == '!':
                        states.append((18, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((19, self.reader.save()))
                        continue
                    if ch == '|':
                        states.append((20, self.reader.save()))
                        continue
                    if ch == '/':
                        states.append((21, self.reader.save()))
                        continue
                    if ch == '%':
                        states.append((22, self.reader.save()))
                        continue
                    if ch == ':':
                        states.append((23, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((24, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((25, self.reader.save()))
                        continue
                    if ch in ('6', '7', '2', '8', '3', '9', '4', '5', '1'):
                        states.append((1, self.reader.save()))
                        continue
                    if ch in ('a', 'q', 'G', 'W', 'b', 'H', 'r', 'X', 'c', 's', 'I', 'Y', 'd', 'J', 't', 'Z', 'e', 'K', 'f', 'v', 'g', 'w', 'M', 'h', 'x', 'N', 'i', 'y', 'O', 'j', 'z', 'P', 'k', 'A', 'Q', 'l', 'B', 'R', 'm', 'C', 'S', 'n', 'D', 'T', 'o', 'E', '_', 'p', 'F', 'V'):
                        states.append((4, self.reader.save()))
                        continue
                    if ch in ('U', 'L'):
                        states.append((10, self.reader.save()))
                        continue
                    if ch in ('~', '[', ']', '(', ')', '?', '{', '}', ';', ','):
                        states.append((16, self.reader.save()))
                        continue
                    if self.other_identifier_start(ch):
                        states.append((4, self.reader.save()))
                        continue
                    break
                case 1:
                    if ch == 'l':
                        states.append((26, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((28, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((29, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((31, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((32, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((34, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((35, self.reader.save()))
                        continue
                    if ch in ('0', '4', '1', '8', '5', '9', '2', '6', '3', '7'):
                        states.append((1, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((27, self.reader.save()))
                        continue
                    break
                case 2:
                    if ch == '<':
                        states.append((36, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 3:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 4:
                    if ch == '\\':
                        states.append((37, self.reader.save()))
                        continue
                    if ch in ('1', '9', 'n', 'f', 'v', 'D', 'L', 'T', '2', 'o', 'g', 'w', 'E', 'M', 'U', '3', '_', 'p', 'h', 'x', 'F', 'N', 'V', '4', 'q', 'a', 'i', 'y', 'G', 'O', 'W', '5', 'b', 'j', 'r', 'z', 'H', 'P', 'X', 's', '6', 'c', 'k', 'A', 'I', 'Q', 'Y', '7', 'd', 'l', 't', 'Z', 'B', 'J', 'R', '0', 'u', '8', 'm', 'e', 'C', 'K', 'S'):
                        states.append((4, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((4, self.reader.save()))
                        continue
                    break
                case 5:
                    if ch == '#':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 6:
                    if ch in ('>', '=', '-'):
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 7:
                    if ch == 'W':
                        states.append((38, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((41, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((44, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((45, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((32, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((34, self.reader.save()))
                        continue
                    if ch in ('0', '4', '1', '5', '3', '7', '2', '6'):
                        states.append((39, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((40, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((42, self.reader.save()))
                        continue
                    if ch in ('X', 'x'):
                        states.append((43, self.reader.save()))
                        continue
                    if ch in ('b', 'B'):
                        states.append((46, self.reader.save()))
                        continue
                    break
                case 8:
                    if ch in ('+', '='):
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 9:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((48, self.reader.save()))
                        continue
                    break
                case 10:
                    if ch == '\\':
                        states.append((37, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((24, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((25, self.reader.save()))
                        continue
                    if ch in ('1', '9', 'n', 'f', 'v', 'D', 'L', 'T', '2', 'o', 'g', 'w', 'E', 'M', 'U', '3', '_', 'p', 'h', 'x', 'F', 'N', 'V', '4', 'q', 'a', 'i', 'y', 'G', 'O', 'W', '5', 'b', 'j', 'r', 'z', 'H', 'P', 'X', 's', '6', 'c', 'k', 'A', 'I', 'Q', 'Y', '7', 'd', 'l', 't', 'Z', 'B', 'J', 'R', '0', 'u', '8', 'm', 'e', 'C', 'K', 'S'):
                        states.append((4, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((4, self.reader.save()))
                        continue
                    break
                case 11:
                    if ch in ('&', '='):
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 12:
                    if ch == '.':
                        states.append((49, self.reader.save()))
                        continue
                    if ch in ('0', '2', '3', '4', '9', '1', '5', '6', '7', '8'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 13:
                    break
                case 14:
                    if ch == 'u':
                        states.append((51, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 15:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 16:
                    break
                case 17:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 18:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 19:
                    if ch == '\\':
                        states.append((37, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((24, self.reader.save()))
                        continue
                    if ch == '8':
                        states.append((10, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((25, self.reader.save()))
                        continue
                    if ch in ('1', '9', 'n', 'f', 'v', 'D', 'L', 'T', '2', 'o', 'g', 'w', 'E', 'M', 'U', '3', '_', 'p', 'h', 'x', 'F', 'N', 'V', '4', 'q', 'a', 'i', 'y', 'G', 'O', 'W', '5', 'b', 'j', 'r', 'z', 'H', 'P', 'X', 's', '6', 'c', 'k', 'A', 'I', 'Q', 'Y', '7', 'd', 'l', 't', 'Z', 'B', 'J', 'R', '0', 'u', 'm', 'e', 'C', 'K', 'S'):
                        states.append((4, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((4, self.reader.save()))
                        continue
                    break
                case 20:
                    if ch in ('|', '='):
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 21:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 22:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 23:
                    if ch == ':':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 24:
                    if ch == '\\':
                        states.append((53, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((55, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 25:
                    if ch == '\\':
                        states.append((56, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 26:
                    if ch == 'l':
                        states.append((59, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 27:
                    if ch == 'l':
                        states.append((60, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((61, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((62, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((63, self.reader.save()))
                        continue
                    break
                case 28:
                    if ch == 'B':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 29:
                    if ch in ('0', '4', '1', '8', '5', '9', '2', '6', '3', '7'):
                        states.append((1, self.reader.save()))
                        continue
                    break
                case 30:
                    if ch in ('+', '-'):
                        states.append((65, self.reader.save()))
                        continue
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((66, self.reader.save()))
                        continue
                    break
                case 31:
                    if ch == 'L':
                        states.append((59, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 32:
                    if ch == 'd':
                        states.append((68, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((69, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((70, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((71, self.reader.save()))
                        continue
                    if ch in ('F', 'f', 'l', 'L'):
                        states.append((67, self.reader.save()))
                        continue
                    if ch in ('0', '2', '4', '6', '8', '1', '7', '9', '3', '5'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 33:
                    break
                case 34:
                    if ch in ('-', '+'):
                        states.append((72, self.reader.save()))
                        continue
                    if ch in ('1', '2', '3', '4', '6', '0', '5', '7', '8', '9'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 35:
                    if ch == 'b':
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 36:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 37:
                    if ch == 'U':
                        states.append((74, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((75, self.reader.save()))
                        continue
                    break
                case 38:
                    if ch == 'B':
                        states.append((76, self.reader.save()))
                        continue
                    break
                case 39:
                    if ch == 'W':
                        states.append((38, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((41, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((44, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((45, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((32, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((34, self.reader.save()))
                        continue
                    if ch in ('0', '4', '1', '5', '3', '7', '2', '6'):
                        states.append((39, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((40, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((42, self.reader.save()))
                        continue
                    break
                case 40:
                    if ch == 'w':
                        states.append((77, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((78, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((79, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((80, self.reader.save()))
                        continue
                    break
                case 41:
                    if ch == 'l':
                        states.append((81, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 42:
                    if ch == '.':
                        states.append((32, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((82, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((34, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((30, self.reader.save()))
                        continue
                    if ch in ('2', '0', '4', '6', '8', '1', '3', '7', '5', '9'):
                        states.append((42, self.reader.save()))
                        continue
                    break
                case 43:
                    if ch == '.':
                        states.append((84, self.reader.save()))
                        continue
                    if ch in ('e', '6', 'B', '1', '9', 'f', '7', '5', '2', 'a', 'C', 'A', '0', '8', 'b', '3', 'D', 'E', 'c', '4', 'd', 'F'):
                        states.append((83, self.reader.save()))
                        continue
                    break
                case 44:
                    if ch == 'L':
                        states.append((81, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 45:
                    if ch in ('0', '4', '1', '5', '3', '7', '2', '6'):
                        states.append((39, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((42, self.reader.save()))
                        continue
                    break
                case 46:
                    if ch in ('0', '1'):
                        states.append((85, self.reader.save()))
                        continue
                    break
                case 47:
                    if ch == 'b':
                        states.append((76, self.reader.save()))
                        continue
                    break
                case 48:
                    if ch == '=':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 49:
                    if ch == '.':
                        states.append((16, self.reader.save()))
                        continue
                    break
                case 50:
                    if ch == "'":
                        states.append((86, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((68, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((69, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((70, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((71, self.reader.save()))
                        continue
                    if ch in ('F', 'L', 'l', 'f'):
                        states.append((67, self.reader.save()))
                        continue
                    if ch in ('2', '0', '4', '6', '8', '1', '3', '9', '7', '5'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 51:
                    if ch in ('2', '0', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'F', '1', '3', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'E'):
                        states.append((87, self.reader.save()))
                        continue
                    break
                case 52:
                    if ch in ('1', '3', '9', 'b', '7', '5', 'd', 'f', 'B', 'D', 'F', '2', '0', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E'):
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 53:
                    if ch == 'U':
                        states.append((89, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((91, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((92, self.reader.save()))
                        continue
                    if ch in ('4', '2', '1', '7', '5', '0', '6', '3'):
                        states.append((90, self.reader.save()))
                        continue
                    if ch in ('"', 'n', '?', 'r', '\\', 't', 'a', 'v', 'b', "'", 'f'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 54:
                    if ch == '\\':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((55, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 55:
                    break
                case 56:
                    if ch == 'u':
                        states.append((95, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('0', '4', '1', '5', '3', '7', '2', '6'):
                        states.append((94, self.reader.save()))
                        continue
                    if ch in ('r', '\\', 'f', '"', 't', 'a', 'n', '?', 'v', 'b', "'"):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 57:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 58:
                    break
                case 59:
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 60:
                    if ch == 'l':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 61:
                    if ch == 'L':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 62:
                    if ch == 'B':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 63:
                    if ch == 'b':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 64:
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 65:
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((66, self.reader.save()))
                        continue
                    break
                case 66:
                    if ch == 'd':
                        states.append((100, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((101, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((102, self.reader.save()))
                        continue
                    if ch in ('5', '7', '1', '9', '2', '4', '0', '6', '8', '3'):
                        states.append((66, self.reader.save()))
                        continue
                    if ch in ('f', 'L', 'F', 'l'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 67:
                    break
                case 68:
                    if ch in ('l', 'd', 'f'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 69:
                    if ch in ('D', 'F', 'L'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 70:
                    if ch in ('4', '1', '0', '5', '8', '9', '6', '7', '2', '3'):
                        states.append((103, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((104, self.reader.save()))
                        continue
                    break
                case 71:
                    if ch in ('4', '9', '5', '6', '7', '8', '1', '2', '3', '0'):
                        states.append((105, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((106, self.reader.save()))
                        continue
                    break
                case 72:
                    if ch in ('1', '2', '3', '4', '6', '0', '5', '7', '8', '9'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 73:
                    if ch == 'd':
                        states.append((100, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((101, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((107, self.reader.save()))
                        continue
                    if ch in ('0', '2', '4', '5', '8', '6', '1', '3', '9', '7'):
                        states.append((73, self.reader.save()))
                        continue
                    if ch in ('L', 'f', 'l', 'F'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 74:
                    if ch in ('a', '2', '0', '4', '6', '8', 'c', 'e', 'A', 'C', 'F', '3', '1', '9', 'd', '5', '7', 'f', 'B', 'b', 'D', 'E'):
                        states.append((108, self.reader.save()))
                        continue
                    break
                case 75:
                    if ch in ('3', '1', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'E', '0', '2', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'F'):
                        states.append((109, self.reader.save()))
                        continue
                    break
                case 76:
                    if ch in ('U', 'u'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 77:
                    if ch == 'b':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 78:
                    if ch == 'B':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 79:
                    if ch == 'l':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 80:
                    if ch == 'L':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 81:
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 82:
                    if ch in ('2', '0', '4', '6', '8', '1', '3', '7', '5', '9'):
                        states.append((42, self.reader.save()))
                        continue
                    break
                case 83:
                    if ch == 'P':
                        states.append((110, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((111, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((112, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((113, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((114, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((116, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((117, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((118, self.reader.save()))
                        continue
                    if ch in ('d', 'F', 'c', '4', '5', 'E', 'A', '0', '8', 'e', '6', '9', 'B', '1', 'f', '7', '2', 'a', 'C', 'b', 'D', '3'):
                        states.append((83, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((115, self.reader.save()))
                        continue
                    break
                case 84:
                    if ch in ('5', '7', 'd', 'f', 'B', 'D', 'F', '3', '9', 'b', '2', '4', 'a', '0', '6', '8', 'c', 'e', 'A', 'C', 'E', '1'):
                        states.append((119, self.reader.save()))
                        continue
                    break
                case 85:
                    if ch == "'":
                        states.append((121, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((122, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((123, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((124, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((125, self.reader.save()))
                        continue
                    if ch in ('0', '1'):
                        states.append((85, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((120, self.reader.save()))
                        continue
                    break
                case 86:
                    if ch in ('2', '0', '1', '3', '7', '4', '5', '6', '8', '9'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 87:
                    if ch in ('b', 'd', 'f', 'B', 'D', 'F', '0', '5', 'a', 'c', 'e', 'A', '8', '4', '6', '2', 'C', 'E', '1', '9', '7', '3'):
                        states.append((126, self.reader.save()))
                        continue
                    break
                case 88:
                    if ch in ('c', 'e', 'A', 'C', 'E', '6', '3', '1', '9', 'b', 'd', 'f', 'B', '5', '7', 'D', 'F', '0', '2', '4', 'a', '8'):
                        states.append((127, self.reader.save()))
                        continue
                    break
                case 89:
                    if ch in ('C', '2', '3', 'a', '1', '4', '9', '5', 'd', 'B', '0', 'F', 'e', '6', 'b', 'f', 'D', '7', 'E', 'c', 'A', '8'):
                        states.append((128, self.reader.save()))
                        continue
                    break
                case 90:
                    if ch == '\\':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('6', '7', '5', '1', '2', '3', '4', '0'):
                        states.append((129, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 91:
                    if ch in ('2', 'a', 'e', 'C', '6', 'B', 'A', 'E', 'D', 'c', '7', 'F', 'd', 'b', '5', '3', 'f', '8', '9', '0', '1', '4'):
                        states.append((130, self.reader.save()))
                        continue
                    break
                case 92:
                    if ch in ('2', 'A', '1', '5', '9', 'd', 'B', 'E', 'F', 'a', 'C', '6', 'e', '0', '7', '3', '4', '8', 'f', 'b', 'D', 'c'):
                        states.append((131, self.reader.save()))
                        continue
                    break
                case 93:
                    if ch == 'U':
                        states.append((133, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((134, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((135, self.reader.save()))
                        continue
                    if ch in ('2', '7', '5', '0', '3', '6', '1', '4'):
                        states.append((132, self.reader.save()))
                        continue
                    if ch in ('n', '?', 'r', '\\', 't', 'a', 'v', 'b', "'", 'f', '"'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 94:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if ch in ('2', '0', '7', '4', '6', '5', '1', '3'):
                        states.append((136, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 95:
                    if ch in ('b', '1', '3', '9', '5', '7', 'd', 'f', 'B', 'D', '0', 'c', '4', '2', 'a', '6', '8', 'e', 'A', 'C', 'E', 'F'):
                        states.append((137, self.reader.save()))
                        continue
                    break
                case 96:
                    if ch in ('0', '2', '4', 'a', 'c', 'A', '6', '8', 'e', 'C', 'E', '1', '3', '9', 'b', 'd', 'f', '5', '7', 'B', 'D', 'F'):
                        states.append((138, self.reader.save()))
                        continue
                    break
                case 97:
                    if ch in ('d', '1', '3', '9', '5', '7', 'f', 'B', 'b', 'D', 'F', '0', '2', '4', 'e', '6', '8', 'A', 'a', 'c', 'C', 'E'):
                        states.append((139, self.reader.save()))
                        continue
                    break
                case 98:
                    if ch == 'u':
                        states.append((141, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((142, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((143, self.reader.save()))
                        continue
                    if ch in ('3', '7', '0', '4', '1', '2', '5', '6'):
                        states.append((140, self.reader.save()))
                        continue
                    if ch in ("'", '\\', 'f', '"', 't', 'r', 'a', 'n', '?', 'v', 'b'):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 99:
                    break
                case 100:
                    if ch in ('l', 'd', 'f'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 101:
                    if ch in ('D', 'L', 'F'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 102:
                    if ch in ('5', '6', '1', '2', '7', '4', '3', '8', '9', '0'):
                        states.append((66, self.reader.save()))
                        continue
                    break
                case 103:
                    if ch == "'":
                        states.append((144, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((68, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((69, self.reader.save()))
                        continue
                    if ch in ('F', 'L', 'l', 'f'):
                        states.append((67, self.reader.save()))
                        continue
                    if ch in ('2', '0', '4', '6', '8', '3', '1', '9', '7', '5'):
                        states.append((103, self.reader.save()))
                        continue
                    break
                case 104:
                    if ch in ('9', '7', '1', '0', '5', '8', '4', '6', '2', '3'):
                        states.append((103, self.reader.save()))
                        continue
                    break
                case 105:
                    if ch == 'd':
                        states.append((68, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((69, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((145, self.reader.save()))
                        continue
                    if ch in ('F', 'L', 'l', 'f'):
                        states.append((67, self.reader.save()))
                        continue
                    if ch in ('4', '0', '2', '6', '8', '1', '3', '9', '5', '7'):
                        states.append((105, self.reader.save()))
                        continue
                    break
                case 106:
                    if ch in ('4', '9', '5', '6', '7', '8', '0', '1', '2', '3'):
                        states.append((105, self.reader.save()))
                        continue
                    break
                case 107:
                    if ch in ('7', '1', '0', '2', '3', '4', '5', '6', '8', '9'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 108:
                    if ch in ('f', 'B', 'D', 'F', '2', '0', '4', 'c', 'e', 'A', '8', '6', '5', 'a', 'C', 'E', '1', '3', '9', 'd', '7', 'b'):
                        states.append((146, self.reader.save()))
                        continue
                    break
                case 109:
                    if ch in ('3', '1', '9', 'b', 'd', 'f', 'B', '5', '7', 'D', 'F', 'c', 'e', 'A', 'C', 'E', '8', '0', '2', '4', 'a', '6'):
                        states.append((147, self.reader.save()))
                        continue
                    break
                case 110:
                    if ch in ('-', '+'):
                        states.append((148, self.reader.save()))
                        continue
                    if ch in ('0', '1', '2', '3', '4', '9', '5', '6', '7', '8'):
                        states.append((149, self.reader.save()))
                        continue
                    break
                case 111:
                    if ch == 'l':
                        states.append((150, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 112:
                    if ch == 'B':
                        states.append((151, self.reader.save()))
                        continue
                    break
                case 113:
                    if ch in ('d', 'F', '5', 'c', '4', 'E', 'A', '0', '8', 'e', '6', 'B', '9', '1', 'f', '7', '2', 'a', 'C', '3', 'b', 'D'):
                        states.append((83, self.reader.save()))
                        continue
                    break
                case 114:
                    if ch == 'L':
                        states.append((150, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 115:
                    if ch == 'l':
                        states.append((152, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((153, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((154, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((155, self.reader.save()))
                        continue
                    break
                case 116:
                    if ch in ('3', '4', '2', '0', '7', '6', '5', '1', '8', '9'):
                        states.append((156, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((157, self.reader.save()))
                        continue
                    break
                case 117:
                    if ch == 'p':
                        states.append((158, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((159, self.reader.save()))
                        continue
                    if ch in ('5', '7', 'd', 'f', 'B', 'D', 'F', '3', '9', 'b', '2', '4', 'a', '0', '6', '8', 'c', 'e', 'A', 'C', 'E', '1'):
                        states.append((119, self.reader.save()))
                        continue
                    break
                case 118:
                    if ch == 'b':
                        states.append((151, self.reader.save()))
                        continue
                    break
                case 119:
                    if ch == "'":
                        states.append((160, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((158, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((159, self.reader.save()))
                        continue
                    if ch in ('A', '0', '2', '4', '6', '8', 'a', 'c', 'e', 'C', 'E', 'B', '3', '1', '9', '5', '7', 'b', 'd', 'f', 'D', 'F'):
                        states.append((119, self.reader.save()))
                        continue
                    break
                case 120:
                    if ch == 'l':
                        states.append((161, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((162, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((163, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((164, self.reader.save()))
                        continue
                    break
                case 121:
                    if ch in ('0', '1'):
                        states.append((85, self.reader.save()))
                        continue
                    break
                case 122:
                    if ch == 'l':
                        states.append((165, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 123:
                    if ch == 'B':
                        states.append((166, self.reader.save()))
                        continue
                    break
                case 124:
                    if ch == 'b':
                        states.append((166, self.reader.save()))
                        continue
                    break
                case 125:
                    if ch == 'L':
                        states.append((165, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 126:
                    if ch in ('1', '3', '9', 'f', 'B', '5', '7', 'b', 'd', 'D', 'F', '2', '4', '0', 'a', 'e', 'A', '6', '8', 'c', 'C', 'E'):
                        states.append((167, self.reader.save()))
                        continue
                    break
                case 127:
                    if ch in ('2', '4', '0', 'a', 'e', 'A', '6', '8', 'c', 'C', 'E', '1', '3', '9', 'b', 'f', 'B', '7', '5', 'd', 'D', 'F'):
                        states.append((168, self.reader.save()))
                        continue
                    break
                case 128:
                    if ch in ('a', '6', '8', 'C', 'D', '5', '7', 'B', 'E', 'c', 'F', 'b', 'f', 'e', '3', '9', 'A', 'd', '4', '1', '0', '2'):
                        states.append((169, self.reader.save()))
                        continue
                    break
                case 129:
                    if ch == '\\':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('0', '4', '1', '5', '6', '7', '2', '3'):
                        states.append((54, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 130:
                    if ch in ('D', 'F', 'B', 'C', 'E', 'A', '4', 'c', '8', '0', '6', '7', 'b', '5', 'a', 'e', 'f', '9', '3', '1', 'd', '2'):
                        states.append((170, self.reader.save()))
                        continue
                    break
                case 131:
                    if ch == '\\':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('1', 'f', '8', '9', 'B', '2', 'a', 'C', '3', 'b', 'D', '6', 'E', '4', 'c', 'A', '0', 'e', 'd', '5', 'F', '7'):
                        states.append((131, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 132:
                    if ch == '\\':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('2', '1', '5', '6', '3', '7', '4', '0'):
                        states.append((171, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 133:
                    if ch in ('5', 'B', '2', 'a', 'e', 'C', '6', '1', 'f', '3', 'b', 'D', '7', 'A', '0', '4', 'c', '8', 'E', '9', 'd', 'F'):
                        states.append((172, self.reader.save()))
                        continue
                    break
                case 134:
                    if ch in ('1', '5', '9', 'd', 'B', 'F', '2', '6', 'a', 'e', 'C', '3', 'f', '7', 'b', 'D', '0', '4', '8', 'c', 'A', 'E'):
                        states.append((173, self.reader.save()))
                        continue
                    break
                case 135:
                    if ch in ('6', 'e', '3', 'b', 'f', 'D', '7', 'c', '0', '4', 'A', '8', 'E', '1', '9', 'd', 'B', '5', 'F', 'a', '2', 'C'):
                        states.append((174, self.reader.save()))
                        continue
                    break
                case 136:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if ch in ('3', '1', '5', '7', '0', '2', '4', '6'):
                        states.append((57, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 137:
                    if ch in ('6', '8', 'c', 'e', 'A', 'C', 'E', '1', '3', '9', '5', '7', 'b', 'd', 'f', 'B', '0', 'D', 'F', '4', 'a', '2'):
                        states.append((175, self.reader.save()))
                        continue
                    break
                case 138:
                    if ch in ('5', '7', 'f', 'B', 'D', 'F', '3', '9', 'b', 'd', '2', '4', 'a', 'c', '6', '8', 'e', 'A', '0', 'C', 'E', '1'):
                        states.append((176, self.reader.save()))
                        continue
                    break
                case 139:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if ch in ('7', 'b', 'D', 'c', 'A', '4', '0', '8', 'E', 'd', 'B', 'F', '1', '5', '9', 'a', 'e', '2', 'C', '6', '3', 'f'):
                        states.append((139, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 140:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if ch in ('3', '1', '0', '5', '7', '4', '6', '2'):
                        states.append((177, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 141:
                    if ch in ('1', '3', '5', '7', '9', 'b', 'd', 'f', 'B', 'D', 'F', '2', '0', '4', '6', '8', 'a', 'c', 'e', 'A', 'C', 'E'):
                        states.append((178, self.reader.save()))
                        continue
                    break
                case 142:
                    if ch in ('2', '0', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E', '3', '1', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'F'):
                        states.append((179, self.reader.save()))
                        continue
                    break
                case 143:
                    if ch in ('1', '9', 'b', 'd', 'f', 'B', 'D', 'F', '0', '2', '4', 'a', 'c', 'e', 'A', '6', '7', '8', 'C', 'E', '3', '5'):
                        states.append((180, self.reader.save()))
                        continue
                    break
                case 144:
                    if ch in ('2', '3', '0', '1', '6', '7', '4', '5', '8', '9'):
                        states.append((103, self.reader.save()))
                        continue
                    break
                case 145:
                    if ch in ('4', '0', '2', '1', '5', '6', '7', '3', '8', '9'):
                        states.append((105, self.reader.save()))
                        continue
                    break
                case 146:
                    if ch in ('b', 'd', '1', '3', '5', '7', '9', 'f', 'B', 'D', 'F', 'a', 'c', '0', '2', '6', '4', '8', 'e', 'A', 'C', 'E'):
                        states.append((181, self.reader.save()))
                        continue
                    break
                case 147:
                    if ch in ('a', '4', '2', '0', 'A', '6', '8', 'c', 'e', 'C', 'E', 'b', '3', '1', '9', 'B', '7', '5', 'd', 'f', 'D', 'F'):
                        states.append((182, self.reader.save()))
                        continue
                    break
                case 148:
                    if ch in ('0', '1', '2', '3', '4', '9', '5', '6', '7', '8'):
                        states.append((149, self.reader.save()))
                        continue
                    break
                case 149:
                    if ch == "'":
                        states.append((183, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((184, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((185, self.reader.save()))
                        continue
                    if ch in ('4', '6', '8', '9', '3', '1', '5', '7', '0', '2'):
                        states.append((149, self.reader.save()))
                        continue
                    if ch in ('F', 'f', 'l', 'L'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 150:
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 151:
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 152:
                    if ch == 'l':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 153:
                    if ch == 'L':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 154:
                    if ch == 'B':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 155:
                    if ch == 'b':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 156:
                    if ch == 'D':
                        states.append((184, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((186, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((185, self.reader.save()))
                        continue
                    if ch in ('8', '1', '3', '9', '5', '7', '2', '0', '4', '6'):
                        states.append((156, self.reader.save()))
                        continue
                    if ch in ('F', 'f', 'L', 'l'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 157:
                    if ch in ('3', '4', '2', '0', '7', '6', '5', '1', '8', '9'):
                        states.append((156, self.reader.save()))
                        continue
                    break
                case 158:
                    if ch in ('4', '7', '1', '0', '5', '8', '9', '6', '2', '3'):
                        states.append((187, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((188, self.reader.save()))
                        continue
                    break
                case 159:
                    if ch in ('4', '9', '5', '6', '7', '8', '1', '2', '3', '0'):
                        states.append((189, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((190, self.reader.save()))
                        continue
                    break
                case 160:
                    if ch in ('A', '0', '2', '4', '6', '8', 'a', 'c', 'e', 'C', 'E', 'B', '3', '1', '9', '5', '7', 'b', 'd', 'f', 'D', 'F'):
                        states.append((119, self.reader.save()))
                        continue
                    break
                case 161:
                    if ch == 'l':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 162:
                    if ch == 'L':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 163:
                    if ch == 'B':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 164:
                    if ch == 'b':
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 165:
                    if ch in ('U', 'u'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 166:
                    if ch in ('u', 'U'):
                        states.append((58, self.reader.save()))
                        continue
                    break
                case 167:
                    if ch in ('6', '8', '4', 'c', 'e', '0', 'A', 'C', 'E', '3', '9', 'b', 'd', '5', '7', 'f', 'B', '1', 'D', 'F', 'a', '2'):
                        states.append((4, self.reader.save()))
                        continue
                    break
                case 168:
                    if ch in ('5', '7', 'b', 'd', 'f', 'B', 'D', 'F', '3', 'a', 'c', 'e', 'A', '6', '8', '0', '2', '4', 'C', 'E', '1', '9'):
                        states.append((191, self.reader.save()))
                        continue
                    break
                case 169:
                    if ch in ('4', '0', 'A', '8', 'c', 'E', '7', '1', '9', '6', 'B', '5', 'd', 'F', 'f', 'C', 'e', 'a', 'D', '2', 'b', '3'):
                        states.append((192, self.reader.save()))
                        continue
                    break
                case 170:
                    if ch in ('D', '1', '5', '9', 'd', 'B', 'F', '7', 'e', 'c', 'b', '6', '8', 'f', '2', 'a', 'C', '3', '0', '4', 'A', 'E'):
                        states.append((193, self.reader.save()))
                        continue
                    break
                case 171:
                    if ch == '\\':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('1', '5', '2', '6', '3', '7', '4', '0'):
                        states.append((54, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 172:
                    if ch in ('b', 'f', 'D', '7', '3', '0', '4', 'c', 'A', 'E', '8', '1', '9', 'd', 'B', 'F', '5', '2', 'a', 'e', 'C', '6'):
                        states.append((194, self.reader.save()))
                        continue
                    break
                case 173:
                    if ch == '\\':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('1', '9', 'B', '2', 'a', 'C', 'b', '3', 'D', '4', 'c', 'E', 'd', '5', 'F', 'e', '6', 'f', '7', 'A', '0', '8'):
                        states.append((173, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 174:
                    if ch in ('4', 'c', 'A', 'E', '8', '0', 'F', '1', '9', 'd', '5', 'B', '2', 'a', 'e', 'C', '6', '3', 'b', 'f', 'D', '7'):
                        states.append((195, self.reader.save()))
                        continue
                    break
                case 175:
                    if ch in ('a', 'c', 'e', '0', '2', '4', '6', '8', 'A', 'C', 'E', 'B', 'b', 'd', '1', '3', '5', '7', '9', 'f', 'D', 'F'):
                        states.append((196, self.reader.save()))
                        continue
                    break
                case 176:
                    if ch in ('b', 'd', 'f', 'B', '1', '3', '5', '7', '9', 'D', 'E', 'a', 'c', 'e', 'A', '0', '2', '4', '6', '8', 'C', 'F'):
                        states.append((197, self.reader.save()))
                        continue
                    break
                case 177:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if ch in ('1', '3', '7', '5', '0', '2', '4', '6'):
                        states.append((57, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 178:
                    if ch in ('0', '2', '4', 'E', 'a', 'c', '1', '3', '9', 'b', '7', 'd', 'f', 'A', 'B', '5', 'C', 'D', 'F', '8', 'e', '6'):
                        states.append((198, self.reader.save()))
                        continue
                    break
                case 179:
                    if ch in ('7', 'B', '2', '4', 'a', 'c', '8', 'e', 'A', '0', '6', 'C', 'E', 'F', '3', '9', 'b', 'd', 'f', '1', 'D', '5'):
                        states.append((199, self.reader.save()))
                        continue
                    break
                case 180:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if ch in ('0', 'c', 'A', '4', 'E', '8', '1', 'd', 'B', 'F', '9', '5', 'f', 'D', '2', 'a', 'e', 'C', '6', '3', 'b', '7'):
                        states.append((180, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 181:
                    if ch in ('6', '8', 'e', 'A', 'C', 'E', 'b', 'd', 'f', 'B', '5', '7', '1', '3', '9', 'D', 'F', '0', '2', '4', 'c', 'a'):
                        states.append((200, self.reader.save()))
                        continue
                    break
                case 182:
                    if ch in ('5', '7', 'C', 'E', '1', '3', 'a', 'c', 'e', 'A', '6', '8', 'b', '2', 'd', '4', 'f', 'B', 'D', 'F', '0', '9'):
                        states.append((4, self.reader.save()))
                        continue
                    break
                case 183:
                    if ch in ('3', '4', '5', '6', '7', '8', '9', '0', '1', '2'):
                        states.append((149, self.reader.save()))
                        continue
                    break
                case 184:
                    if ch in ('D', 'L', 'F'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 185:
                    if ch in ('f', 'd', 'l'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 186:
                    if ch in ('6', '7', '8', '1', '2', '3', '0', '5', '4', '9'):
                        states.append((156, self.reader.save()))
                        continue
                    break
                case 187:
                    if ch == 'd':
                        states.append((201, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((202, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((203, self.reader.save()))
                        continue
                    if ch in ('2', '0', '4', '6', '8', '3', '1', '9', '7', '5'):
                        states.append((187, self.reader.save()))
                        continue
                    if ch in ('L', 'l', 'f', 'F'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 188:
                    if ch in ('9', '1', '0', '6', '5', '4', '8', '7', '2', '3'):
                        states.append((187, self.reader.save()))
                        continue
                    break
                case 189:
                    if ch == 'd':
                        states.append((201, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((203, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((204, self.reader.save()))
                        continue
                    if ch in ('L', 'l', 'f', 'F'):
                        states.append((67, self.reader.save()))
                        continue
                    if ch in ('4', '0', '2', '6', '8', '1', '3', '9', '5', '7'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 190:
                    if ch in ('4', '9', '5', '6', '7', '8', '0', '1', '2', '3'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 191:
                    if ch in ('A', '2', '0', '4', '6', '8', 'a', 'c', 'e', 'C', 'F', '5', 'd', 'f', 'B', '3', '7', '1', '9', 'b', 'D', 'E'):
                        states.append((205, self.reader.save()))
                        continue
                    break
                case 192:
                    if ch in ('5', 'D', '9', 'F', 'B', 'a', 'e', '2', 'C', '6', 'E', 'b', 'f', '3', '7', '1', 'A', 'c', '0', '4', '8', 'd'):
                        states.append((206, self.reader.save()))
                        continue
                    break
                case 193:
                    if ch in ('6', 'a', 'e', 'C', 'E', 'A', 'f', 'D', 'd', 'c', '3', 'b', 'F', '7', 'B', '8', '9', '5', '4', '0', '1', '2'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 194:
                    if ch in ('0', '4', 'A', 'c', '8', 'E', '1', '9', 'B', '5', 'd', 'F', '2', 'a', 'e', '6', 'C', '3', 'b', 'f', '7', 'D'):
                        states.append((207, self.reader.save()))
                        continue
                    break
                case 195:
                    if ch in ('1', '9', 'd', '5', 'B', 'F', '2', 'e', '6', 'a', 'C', '3', 'f', '7', 'b', 'D', '0', '4', 'A', '8', 'c', 'E'):
                        states.append((208, self.reader.save()))
                        continue
                    break
                case 196:
                    if ch in ('3', '1', '9', 'b', '7', '5', 'd', 'f', 'B', 'D', 'F', '0', '2', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E'):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 197:
                    if ch in ('a', '2', '0', '4', '6', '8', 'c', 'A', 'e', 'C', 'F', '3', '1', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'E'):
                        states.append((209, self.reader.save()))
                        continue
                    break
                case 198:
                    if ch in ('0', '4', '2', 'a', '6', '8', 'c', 'e', 'A', 'C', 'F', 'B', '1', '3', '9', '5', '7', 'b', 'd', 'f', 'D', 'E'):
                        states.append((210, self.reader.save()))
                        continue
                    break
                case 199:
                    if ch in ('B', '3', '1', '9', '5', '7', 'd', 'f', 'b', 'D', 'E', '0', '2', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'F'):
                        states.append((211, self.reader.save()))
                        continue
                    break
                case 200:
                    if ch in ('1', '3', 'B', '9', 'b', 'd', '7', 'f', '5', 'D', 'F', '0', '2', '4', 'A', 'a', 'c', 'e', '8', 'C', 'E', '6'):
                        states.append((212, self.reader.save()))
                        continue
                    break
                case 201:
                    if ch in ('d', 'l', 'f'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 202:
                    if ch in ('2', '3', '0', '1', '6', '7', '4', '5', '8', '9'):
                        states.append((187, self.reader.save()))
                        continue
                    break
                case 203:
                    if ch in ('L', 'F', 'D'):
                        states.append((67, self.reader.save()))
                        continue
                    break
                case 204:
                    if ch in ('4', '0', '1', '2', '5', '6', '7', '3', '8', '9'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 205:
                    if ch in ('1', '3', '5', '7', '9', 'b', 'd', 'f', 'B', 'D', 'F', '2', '0', '4', '6', '8', 'a', 'c', 'e', 'A', 'C', 'E'):
                        states.append((213, self.reader.save()))
                        continue
                    break
                case 206:
                    if ch in ('C', 'e', '2', 'a', '6', '1', '7', '8', '4', 'D', 'E', 'f', '3', 'b', 'F', '5', 'A', '0', 'c', 'd', 'B', '9'):
                        states.append((214, self.reader.save()))
                        continue
                    break
                case 207:
                    if ch in ('2', 'a', 'e', 'C', '6', '3', 'b', 'f', 'D', '7', '0', '4', 'c', 'A', '8', 'E', 'F', '1', '9', 'd', '5', 'B'):
                        states.append((215, self.reader.save()))
                        continue
                    break
                case 208:
                    if ch in ('3', 'b', 'f', '7', 'D', '0', '4', 'c', 'A', '8', 'E', '1', '9', 'd', 'B', '5', 'F', '2', 'a', 'e', 'C', '6'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 209:
                    if ch in ('6', '8', 'a', 'c', '1', '3', 'A', 'C', '5', 'f', '9', 'b', 'd', '7', 'B', 'D', 'E', 'F', '4', '2', '0', 'e'):
                        states.append((216, self.reader.save()))
                        continue
                    break
                case 210:
                    if ch in ('9', 'b', 'd', 'f', '5', '7', 'B', 'D', 'F', '0', '2', '4', 'a', 'c', 'e', 'A', '6', '8', 'C', 'E', '3', '1'):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 211:
                    if ch in ('6', 'a', 'c', 'e', '8', 'A', 'C', 'E', '1', '3', '5', '7', '9', 'b', 'd', 'f', 'B', 'D', 'F', '2', '4', '0'):
                        states.append((217, self.reader.save()))
                        continue
                    break
                case 212:
                    if ch in ('0', '2', '4', 'a', '6', '8', 'e', 'A', 'c', 'C', 'E', '1', '3', '9', 'b', '5', '7', 'f', 'B', 'd', 'D', 'F'):
                        states.append((218, self.reader.save()))
                        continue
                    break
                case 213:
                    if ch in ('0', '2', '4', 'E', 'a', 'c', '1', '3', '9', 'b', '7', 'd', 'f', 'A', 'B', '5', 'C', 'D', 'F', '8', 'e', '6'):
                        states.append((219, self.reader.save()))
                        continue
                    break
                case 214:
                    if ch in ('5', '3', 'b', 'f', 'D', '7', 'E', 'c', 'e', '0', '4', 'A', '8', '2', 'd', 'F', '1', '9', 'B', 'C', '6', 'a'):
                        states.append((220, self.reader.save()))
                        continue
                    break
                case 215:
                    if ch in ('6', '2', 'a', 'e', 'C', 'D', 'f', '3', 'b', '7', '0', '4', 'A', 'c', '8', 'E', 'F', 'd', 'B', '1', '5', '9'):
                        states.append((221, self.reader.save()))
                        continue
                    break
                case 216:
                    if ch in ('A', '0', '2', '4', '6', '8', 'a', 'c', 'e', 'C', 'E', 'B', '3', '1', '9', '5', '7', 'b', 'd', 'f', 'D', 'F'):
                        states.append((222, self.reader.save()))
                        continue
                    break
                case 217:
                    if ch in ('d', 'f', '1', '3', 'B', '7', '5', '8', '9', 'b', 'C', 'D', 'E', 'F', 'c', 'e', 'A', '2', '6', '0', '4', 'a'):
                        states.append((223, self.reader.save()))
                        continue
                    break
                case 218:
                    if ch in ('3', 'D', '9', 'b', 'd', 'f', 'B', '0', '2', '4', 'a', '8', 'c', 'e', 'A', '6', '5', 'F', 'C', '1', 'E', '7'):
                        states.append((224, self.reader.save()))
                        continue
                    break
                case 219:
                    if ch in ('0', '4', '2', 'a', '6', '8', 'c', 'e', 'A', 'C', 'F', '1', '3', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'E'):
                        states.append((4, self.reader.save()))
                        continue
                    break
                case 220:
                    if ch in ('1', '5', '9', 'd', 'B', 'F', '2', 'e', '6', 'a', 'C', 'E', 'c', '3', 'f', '7', 'b', 'D', '0', '4', 'A', '8'):
                        states.append((225, self.reader.save()))
                        continue
                    break
                case 221:
                    if ch in ('7', 'D', '0', '4', 'c', 'A', 'E', '8', '3', '1', '5', '9', 'd', 'B', 'F', 'f', '2', '6', 'a', 'e', 'C', 'b'):
                        states.append((226, self.reader.save()))
                        continue
                    break
                case 222:
                    if ch in ('1', '3', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'F', '0', '2', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E'):
                        states.append((227, self.reader.save()))
                        continue
                    break
                case 223:
                    if ch in ('E', 'c', 'e', 'A', '2', '4', '0', '6', '8', 'a', 'C', 'F', 'd', 'f', 'B', '1', '3', '5', '7', '9', 'b', 'D'):
                        states.append((228, self.reader.save()))
                        continue
                    break
                case 224:
                    if ch in ('b', '1', '3', '9', '5', '7', 'd', 'B', 'f', 'D', '0', '4', '2', 'a', 'c', '6', '8', 'e', 'A', 'C', 'E', 'F'):
                        states.append((4, self.reader.save()))
                        continue
                    break
                case 225:
                    if ch in ('6', 'a', 'e', '2', 'C', '3', 'b', 'f', '7', 'D', 'c', 'A', '4', '0', '8', 'E', '1', '9', 'd', 'B', '5', 'F'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 226:
                    if ch in ('1', '9', 'd', '5', 'B', 'F', '2', 'e', 'a', '6', 'C', 'b', '3', 'f', '7', 'D', '0', '4', 'c', 'A', '8', 'E'):
                        states.append((229, self.reader.save()))
                        continue
                    break
                case 227:
                    if ch in ('1', '3', 'B', '9', 'b', 'd', '7', 'f', '5', 'D', 'F', '6', '0', '2', '4', 'A', 'a', 'c', 'e', '8', 'C', 'E'):
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 228:
                    if ch in ('5', '7', 'D', 'F', '0', '2', 'e', 'a', 'c', '4', '6', '8', 'A', 'C', 'E', '3', 'f', '9', 'b', '1', 'd', 'B'):
                        states.append((230, self.reader.save()))
                        continue
                    break
                case 229:
                    if ch in ('2', 'a', 'e', 'C', '6', 'f', '3', 'b', 'D', '7', '0', '4', 'c', 'A', '8', 'E', 'B', 'd', '1', '9', '5', 'F'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 230:
                    if ch in ('b', '1', '3', '9', 'B', '5', '7', 'd', 'f', 'D', 'F', 'c', '0', '2', '4', '6', '8', 'e', 'A', 'a', 'C', 'E'):
                        states.append((57, self.reader.save()))
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
                    return Token(TokenKind.INTCONST, location, text)
                case 2:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 3:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 4:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 5:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 6:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 7:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 8:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 9:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 10:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 11:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 12:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 13:
                    self.reader.restore(back_index)
                    return Token(TokenKind.END, location, text)
                case 15:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 16:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 17:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 18:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 19:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
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
                    return self.error('字符串未终止', location)
                case 26:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 27:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 31:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 32:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 36:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 39:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 40:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 41:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 44:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 48:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 50:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 53:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 54:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 55:
                    self.reader.restore(back_index)
                    return Token(TokenKind.STRINGLITERAL, location, text)
                case 56:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 58:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 59:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 60:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 61:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 64:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 66:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 67:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 73:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 76:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 79:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 80:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 81:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 83:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 85:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 90:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 93:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 98:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 99:
                    self.reader.restore(back_index)
                    return Token(TokenKind.CHARCONST, location, text)
                case 103:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 105:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 111:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 114:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 115:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 120:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 122:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 125:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 129:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 131:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 132:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 149:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
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
                case 161:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 162:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 165:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 166:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 171:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 173:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 187:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 189:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
        self.reader.restore(start_index)
        return None