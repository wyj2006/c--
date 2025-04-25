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
                        states.append((1, self.reader.save()))
                        continue
                    if ch == '|':
                        states.append((4, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((7, self.reader.save()))
                        continue
                    if ch == '-':
                        states.append((8, self.reader.save()))
                        continue
                    if ch == ':':
                        states.append((9, self.reader.save()))
                        continue
                    if ch == '^':
                        states.append((10, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((11, self.reader.save()))
                        continue
                    if ch == '0':
                        states.append((12, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((13, self.reader.save()))
                        continue
                    if ch == '+':
                        states.append((14, self.reader.save()))
                        continue
                    if ch == '*':
                        states.append((15, self.reader.save()))
                        continue
                    if ch == '/':
                        states.append((16, self.reader.save()))
                        continue
                    if ch == '%':
                        states.append((17, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((18, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((19, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((20, self.reader.save()))
                        continue
                    if ch == '&':
                        states.append((21, self.reader.save()))
                        continue
                    if ch == '#':
                        states.append((22, self.reader.save()))
                        continue
                    if ch == '':
                        states.append((23, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((24, self.reader.save()))
                        continue
                    if ch == '!':
                        states.append((25, self.reader.save()))
                        continue
                    if ch in ('h', 'x', 'N', 'i', 'y', 'O', 'j', 'z', 'P', 'k', 'A', 'Q', 'l', 'B', 'R', 'm', 'C', 'S', 'n', 'D', 'T', 'o', 'E', '_', 'p', 'F', 'V', 'a', 'q', 'G', 'W', 'b', 'r', 'H', 'X', 'c', 's', 'I', 'Y', 'd', 't', 'J', 'Z', 'e', 'K', 'v', 'f', 'w', 'g', 'M'):
                        states.append((2, self.reader.save()))
                        continue
                    if ch in ('4', '1', '9', '5', '2', '6', '3', '7', '8'):
                        states.append((3, self.reader.save()))
                        continue
                    if ch in ('L', 'U'):
                        states.append((5, self.reader.save()))
                        continue
                    if ch in ('[', ']', '(', ')', '?', '{', '}', ';', ',', '~'):
                        states.append((6, self.reader.save()))
                        continue
                    if self.other_identifier_start(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 1:
                    if ch == '<':
                        states.append((26, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 2:
                    if ch == '\\':
                        states.append((28, self.reader.save()))
                        continue
                    if ch in ('8', 'm', 'u', 'e', 'C', 'K', 'S', '1', '9', 'v', 'n', 'f', 'D', 'L', 'T', '2', 'o', 'w', 'g', 'E', 'M', 'U', '3', 'p', 'x', '_', 'h', 'F', 'N', 'V', '4', 'a', 'i', 'q', 'y', 'G', 'O', 'W', '5', 'r', 'z', 'b', 'j', 'H', 'P', 'X', '6', 'c', 'k', 's', 'A', 'I', 'Q', 'Y', '7', 't', 'd', 'l', 'B', 'J', 'R', 'Z', '0'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 3:
                    if ch == 'e':
                        states.append((29, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((31, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((32, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((33, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((34, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((35, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((36, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((37, self.reader.save()))
                        continue
                    if ch in ('3', '7', '0', '4', '8', '1', '9', '5', '2', '6'):
                        states.append((3, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((30, self.reader.save()))
                        continue
                    break
                case 4:
                    if ch in ('=', '|'):
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 5:
                    if ch == '\\':
                        states.append((28, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((11, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((20, self.reader.save()))
                        continue
                    if ch in ('8', 'm', 'u', 'e', 'C', 'K', 'S', '1', '9', 'v', 'n', 'f', 'D', 'L', 'T', '2', 'o', 'w', 'g', 'E', 'M', 'U', '3', 'p', 'x', '_', 'h', 'F', 'N', 'V', '4', 'a', 'i', 'q', 'y', 'G', 'O', 'W', '5', 'r', 'z', 'b', 'j', 'H', 'P', 'X', '6', 'c', 'k', 's', 'A', 'I', 'Q', 'Y', '7', 't', 'd', 'l', 'B', 'J', 'R', 'Z', '0'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 6:
                    break
                case 7:
                    if ch == '>':
                        states.append((38, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 8:
                    if ch in ('-', '=', '>'):
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 9:
                    if ch == ':':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 10:
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 11:
                    if ch == '\\':
                        states.append((39, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((41, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 12:
                    if ch == 'e':
                        states.append((29, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((46, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((48, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((50, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((35, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((37, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((51, self.reader.save()))
                        continue
                    if ch in ('B', 'b'):
                        states.append((42, self.reader.save()))
                        continue
                    if ch in ('3', '0', '4', '7', '2', '6', '1', '5'):
                        states.append((43, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((44, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((45, self.reader.save()))
                        continue
                    if ch in ('X', 'x'):
                        states.append((49, self.reader.save()))
                        continue
                    break
                case 13:
                    if ch == '8':
                        states.append((5, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((28, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((11, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((20, self.reader.save()))
                        continue
                    if ch in ('m', 'u', 'e', 'C', 'K', 'S', '1', '9', 'v', 'n', 'f', 'D', 'L', 'T', '2', 'o', 'w', 'g', 'E', 'M', 'U', '3', 'p', 'x', '_', 'h', 'F', 'N', 'V', '4', 'a', 'i', 'q', 'y', 'G', 'O', 'W', '5', 'r', 'z', 'b', 'j', 'H', 'P', 'X', '6', 'c', 'k', 's', 'A', 'I', 'Q', 'Y', '7', 't', 'd', 'l', 'B', 'J', 'R', 'Z', '0'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 14:
                    if ch in ('=', '+'):
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 15:
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 16:
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 17:
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 18:
                    if ch == '.':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('6', '7', '8', '0', '1', '2', '3', '4', '9', '5'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 19:
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 20:
                    if ch == '\\':
                        states.append((54, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 21:
                    if ch in ('&', '='):
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 22:
                    if ch == '#':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 23:
                    break
                case 24:
                    if ch == 'u':
                        states.append((56, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((57, self.reader.save()))
                        continue
                    break
                case 25:
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 26:
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 27:
                    break
                case 28:
                    if ch == 'U':
                        states.append((58, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 29:
                    if ch in ('0', '1', '2', '3', '4', '9', '6', '7', '5', '8'):
                        states.append((60, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((61, self.reader.save()))
                        continue
                    break
                case 30:
                    if ch == 'L':
                        states.append((62, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((63, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((64, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 31:
                    if ch == 'b':
                        states.append((66, self.reader.save()))
                        continue
                    break
                case 32:
                    if ch == 'l':
                        states.append((67, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 33:
                    if ch == 'B':
                        states.append((66, self.reader.save()))
                        continue
                    break
                case 34:
                    if ch in ('3', '0', '4', '7', '8', '1', '9', '5', '2', '6'):
                        states.append((3, self.reader.save()))
                        continue
                    break
                case 35:
                    if ch in ('+', '-'):
                        states.append((69, self.reader.save()))
                        continue
                    if ch in ('0', '1', '2', '3', '4', '9', '5', '6', '7', '8'):
                        states.append((70, self.reader.save()))
                        continue
                    break
                case 36:
                    if ch == 'L':
                        states.append((67, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 37:
                    if ch == 'D':
                        states.append((71, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((72, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((74, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((75, self.reader.save()))
                        continue
                    if ch in ('6', '1', '3', '9', '5', '7', '0', '2', '4', '8'):
                        states.append((52, self.reader.save()))
                        continue
                    if ch in ('l', 'F', 'f', 'L'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 38:
                    if ch == '=':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 39:
                    if ch == 'u':
                        states.append((77, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((78, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((79, self.reader.save()))
                        continue
                    if ch in ('3', '1', '6', '7', '4', '5', '2', '0'):
                        states.append((76, self.reader.save()))
                        continue
                    if ch in ('f', '"', 'n', '?', 'r', '\\', 't', 'a', 'v', 'b', "'"):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 40:
                    if ch == '\\':
                        states.append((80, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((41, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 41:
                    break
                case 42:
                    if ch in ('0', '1'):
                        states.append((81, self.reader.save()))
                        continue
                    break
                case 43:
                    if ch == 'e':
                        states.append((29, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((46, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((48, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((50, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((35, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((37, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((51, self.reader.save()))
                        continue
                    if ch in ('3', '7', '0', '4', '2', '6', '1', '5'):
                        states.append((43, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((44, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((45, self.reader.save()))
                        continue
                    break
                case 44:
                    if ch == "'":
                        states.append((82, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((29, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((35, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((37, self.reader.save()))
                        continue
                    if ch in ('1', '3', '9', '2', '0', '4', '5', '6', '8', '7'):
                        states.append((44, self.reader.save()))
                        continue
                    break
                case 45:
                    if ch == 'l':
                        states.append((83, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((84, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((85, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((86, self.reader.save()))
                        continue
                    break
                case 46:
                    if ch == 'l':
                        states.append((87, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 47:
                    if ch in ('3', '0', '4', '7', '2', '6', '1', '5'):
                        states.append((43, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((44, self.reader.save()))
                        continue
                    break
                case 48:
                    if ch == 'B':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 49:
                    if ch == '.':
                        states.append((90, self.reader.save()))
                        continue
                    if ch in ('0', 'A', '8', 'e', '6', 'd', '5', 'F', '1', '9', 'B', 'f', '7', 'a', '2', 'C', '3', 'b', 'D', '4', 'c', 'E'):
                        states.append((89, self.reader.save()))
                        continue
                    break
                case 50:
                    if ch == 'L':
                        states.append((87, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 51:
                    if ch == 'b':
                        states.append((88, self.reader.save()))
                        continue
                    break
                case 52:
                    if ch == 'D':
                        states.append((71, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((72, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((91, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((74, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((75, self.reader.save()))
                        continue
                    if ch in ('1', '3', '9', '5', '7', '0', '2', '4', '6', '8'):
                        states.append((52, self.reader.save()))
                        continue
                    if ch in ('F', 'f', 'L', 'l'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 53:
                    if ch == '.':
                        states.append((6, self.reader.save()))
                        continue
                    break
                case 54:
                    if ch == 'x':
                        states.append((93, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((94, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((95, self.reader.save()))
                        continue
                    if ch in ('7', '2', '6', '0', '1', '4', '5', '3'):
                        states.append((92, self.reader.save()))
                        continue
                    if ch in ('b', "'", 'r', '\\', '?', 'f', '"', 't', 'a', 'n', 'v'):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 55:
                    if ch == '\\':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((97, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 56:
                    if ch in ('1', '3', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'F', '0', '2', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E'):
                        states.append((98, self.reader.save()))
                        continue
                    break
                case 57:
                    if ch in ('4', 'a', 'c', 'e', '6', '8', 'A', 'C', 'E', '2', '1', '3', '9', 'b', 'd', '5', '7', 'f', 'B', 'D', 'F', '0'):
                        states.append((99, self.reader.save()))
                        continue
                    break
                case 58:
                    if ch in ('5', '9', 'b', 'd', '7', 'f', 'B', 'D', 'F', '3', '2', '4', '0', '6', 'a', 'c', 'e', '8', 'A', 'C', 'E', '1'):
                        states.append((100, self.reader.save()))
                        continue
                    break
                case 59:
                    if ch in ('a', 'c', 'e', 'A', '6', '8', 'C', 'E', '2', '4', '1', '3', '9', 'b', 'd', 'f', '5', '7', 'B', 'D', 'F', '0'):
                        states.append((101, self.reader.save()))
                        continue
                    break
                case 60:
                    if ch == 'D':
                        states.append((102, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((103, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((104, self.reader.save()))
                        continue
                    if ch in ('9', '1', '3', '5', '7', '0', '2', '4', '6', '8'):
                        states.append((60, self.reader.save()))
                        continue
                    if ch in ('f', 'F', 'l', 'L'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 61:
                    if ch in ('0', '1', '2', '3', '4', '9', '6', '7', '5', '8'):
                        states.append((60, self.reader.save()))
                        continue
                    break
                case 62:
                    if ch == 'L':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 63:
                    if ch == 'l':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 64:
                    if ch == 'B':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 65:
                    if ch == 'b':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 66:
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 67:
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 68:
                    break
                case 69:
                    if ch in ('0', '1', '2', '3', '4', '9', '5', '6', '7', '8'):
                        states.append((70, self.reader.save()))
                        continue
                    break
                case 70:
                    if ch == 'D':
                        states.append((102, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((103, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((105, self.reader.save()))
                        continue
                    if ch in ('0', '4', '1', '3', '9', '5', '7', '8', '2', '6'):
                        states.append((70, self.reader.save()))
                        continue
                    if ch in ('f', 'F', 'l', 'L'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 71:
                    if ch in ('D', 'L', 'F'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 72:
                    if ch in ('f', 'd', 'l'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 73:
                    break
                case 74:
                    if ch in ('+', '-'):
                        states.append((106, self.reader.save()))
                        continue
                    if ch in ('3', '0', '1', '2', '4', '5', '6', '7', '8', '9'):
                        states.append((107, self.reader.save()))
                        continue
                    break
                case 75:
                    if ch in ('6', '7', '8', '1', '2', '3', '4', '0', '9', '5'):
                        states.append((108, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((109, self.reader.save()))
                        continue
                    break
                case 76:
                    if ch == '\\':
                        states.append((80, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((41, self.reader.save()))
                        continue
                    if ch in ('1', '0', '4', '5', '2', '6', '7', '3'):
                        states.append((110, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 77:
                    if ch in ('1', '9', 'd', 'B', '5', 'F', '4', '6', '2', 'a', 'e', 'C', '3', 'b', 'f', 'D', '7', 'E', 'A', '8', '0', 'c'):
                        states.append((111, self.reader.save()))
                        continue
                    break
                case 78:
                    if ch in ('4', 'c', 'A', 'E', '8', 'F', 'e', 'f', '1', 'd', '9', '5', 'C', 'B', '7', '6', 'D', '0', '3', 'b', '2', 'a'):
                        states.append((112, self.reader.save()))
                        continue
                    break
                case 79:
                    if ch in ('c', '0', '4', 'A', '8', 'E', '6', '7', '5', 'B', 'C', '9', 'f', 'D', 'F', 'b', 'd', 'e', '3', '1', 'a', '2'):
                        states.append((113, self.reader.save()))
                        continue
                    break
                case 80:
                    if ch == 'u':
                        states.append((115, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((116, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((117, self.reader.save()))
                        continue
                    if ch in ('6', '4', '1', '2', '7', '5', '3', '0'):
                        states.append((114, self.reader.save()))
                        continue
                    if ch in ('"', 'f', 'n', '?', 'r', '\\', 't', 'a', 'v', 'b', "'"):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 81:
                    if ch == 'L':
                        states.append((119, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((120, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((121, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((122, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((123, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((118, self.reader.save()))
                        continue
                    if ch in ('0', '1'):
                        states.append((81, self.reader.save()))
                        continue
                    break
                case 82:
                    if ch in ('1', '3', '9', '2', '0', '4', '5', '6', '8', '7'):
                        states.append((44, self.reader.save()))
                        continue
                    break
                case 83:
                    if ch == 'l':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 84:
                    if ch == 'L':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 85:
                    if ch == 'B':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 86:
                    if ch == 'b':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 87:
                    if ch in ('U', 'u'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 88:
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 89:
                    if ch == '.':
                        states.append((124, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((125, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((126, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((127, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((128, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((129, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((130, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((131, self.reader.save()))
                        continue
                    if ch in ('c', 'E', '3', 'b', 'D', '4', 'f', '7', 'd', 'F', '5', '0', 'A', '8', 'e', '6', '1', 'B', '9', '2', 'a', 'C'):
                        states.append((89, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((132, self.reader.save()))
                        continue
                    break
                case 90:
                    if ch in ('0', '4', 'a', 'c', 'B', '1', '3', '9', '7', 'b', 'd', 'f', 'A', 'e', '5', 'C', 'D', 'F', '8', 'E', '2', '6'):
                        states.append((133, self.reader.save()))
                        continue
                    break
                case 91:
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 92:
                    if ch == '\\':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('0', '4', '1', '3', '2', '5', '7', '6'):
                        states.append((134, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 93:
                    if ch in ('6', '8', 'c', 'e', 'A', 'C', 'E', '9', 'b', 'd', 'f', '5', '7', 'B', '1', '3', 'D', 'F', '0', '2', '4', 'a'):
                        states.append((135, self.reader.save()))
                        continue
                    break
                case 94:
                    if ch in ('2', '0', '4', '6', 'a', 'c', 'e', '8', 'A', 'C', 'E', '3', '1', '5', '7', '9', 'b', 'd', 'f', 'B', 'D', 'F'):
                        states.append((136, self.reader.save()))
                        continue
                    break
                case 95:
                    if ch in ('1', '3', '9', '5', '7', 'b', 'd', 'f', 'B', 'D', 'F', '2', '0', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E'):
                        states.append((137, self.reader.save()))
                        continue
                    break
                case 96:
                    if ch == 'x':
                        states.append((139, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((140, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((141, self.reader.save()))
                        continue
                    if ch in ('6', '0', '4', '1', '5', '3', '7', '2'):
                        states.append((138, self.reader.save()))
                        continue
                    if ch in ('b', "'", 'r', '\\', 'f', '"', 't', 'a', 'n', '?', 'v'):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 97:
                    break
                case 98:
                    if ch in ('3', '1', 'B', 'f', '5', '7', '9', 'b', 'd', 'D', 'F', '0', '2', '4', 'A', 'e', '6', '8', 'a', 'c', 'C', 'E'):
                        states.append((142, self.reader.save()))
                        continue
                    break
                case 99:
                    if ch in ('0', '2', '4', 'a', 'e', 'A', '6', '8', 'c', 'C', 'E', '1', '3', '9', 'b', 'f', 'B', '5', '7', 'd', 'D', 'F'):
                        states.append((143, self.reader.save()))
                        continue
                    break
                case 100:
                    if ch in ('d', 'b', '3', '1', 'B', '5', '7', '9', 'f', 'D', 'F', 'a', '0', '2', '4', '6', '8', 'c', 'e', 'A', 'C', 'E'):
                        states.append((144, self.reader.save()))
                        continue
                    break
                case 101:
                    if ch in ('0', '2', '4', 'a', 'e', 'A', '6', '8', 'c', 'C', 'E', '1', '3', '9', 'b', 'f', 'B', '5', '7', 'd', 'D', 'F'):
                        states.append((145, self.reader.save()))
                        continue
                    break
                case 102:
                    if ch in ('F', 'D', 'L'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 103:
                    if ch in ('l', 'f', 'd'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 104:
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((60, self.reader.save()))
                        continue
                    break
                case 105:
                    if ch in ('5', '7', '8', '0', '1', '2', '3', '9', '4', '6'):
                        states.append((70, self.reader.save()))
                        continue
                    break
                case 106:
                    if ch in ('3', '0', '1', '2', '4', '5', '6', '7', '8', '9'):
                        states.append((107, self.reader.save()))
                        continue
                    break
                case 107:
                    if ch == 'D':
                        states.append((71, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((72, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((146, self.reader.save()))
                        continue
                    if ch in ('1', '3', '9', '5', '7', '0', '2', '4', '6', '8'):
                        states.append((107, self.reader.save()))
                        continue
                    if ch in ('F', 'f', 'L', 'l'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 108:
                    if ch == 'D':
                        states.append((71, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((72, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((147, self.reader.save()))
                        continue
                    if ch in ('1', '3', '9', '7', '5', '2', '4', '0', '6', '8'):
                        states.append((108, self.reader.save()))
                        continue
                    if ch in ('F', 'f', 'L', 'l'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 109:
                    if ch in ('5', '7', '8', '0', '2', '3', '4', '1', '9', '6'):
                        states.append((108, self.reader.save()))
                        continue
                    break
                case 110:
                    if ch == '\\':
                        states.append((80, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((41, self.reader.save()))
                        continue
                    if ch in ('3', '7', '4', '0', '1', '2', '5', '6'):
                        states.append((40, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 111:
                    if ch in ('2', '0', '3', 'f', 'b', 'D', '7', '1', '4', 'A', 'c', 'E', '8', 'F', 'B', '5', '9', 'd', 'C', 'e', '6', 'a'):
                        states.append((148, self.reader.save()))
                        continue
                    break
                case 112:
                    if ch in ('2', 'e', '6', 'a', 'C', '3', 'f', '7', 'b', 'D', 'E', 'F', '8', 'B', '0', '4', 'A', 'c', '9', 'd', '5', '1'):
                        states.append((149, self.reader.save()))
                        continue
                    break
                case 113:
                    if ch == '\\':
                        states.append((80, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((41, self.reader.save()))
                        continue
                    if ch in ('A', '0', '8', '7', '5', 'F', 'B', '1', '9', 'D', 'f', 'd', 'b', '3', 'a', '2', 'C', 'c', '4', 'E', '6', 'e'):
                        states.append((113, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 114:
                    if ch == '\\':
                        states.append((80, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((41, self.reader.save()))
                        continue
                    if ch in ('1', '0', '4', '5', '2', '6', '3', '7'):
                        states.append((150, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 115:
                    if ch in ('2', 'a', 'e', 'C', '6', '3', 'b', 'f', 'D', '7', '9', 'A', '4', '0', 'c', '8', 'E', 'B', 'F', '1', '5', 'd'):
                        states.append((151, self.reader.save()))
                        continue
                    break
                case 116:
                    if ch in ('1', '9', 'd', 'B', '5', 'F', '2', 'a', 'e', 'C', '6', '8', '3', 'b', 'f', 'D', '7', 'c', '0', '4', 'A', 'E'):
                        states.append((152, self.reader.save()))
                        continue
                    break
                case 117:
                    if ch in ('8', 'E', 'A', '0', '4', 'c', '1', '9', 'd', '5', 'B', 'F', 'a', 'e', '2', '6', 'C', '3', 'b', 'f', '7', 'D'):
                        states.append((153, self.reader.save()))
                        continue
                    break
                case 118:
                    if ch == 'w':
                        states.append((154, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((155, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((156, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((157, self.reader.save()))
                        continue
                    break
                case 119:
                    if ch == 'L':
                        states.append((158, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 120:
                    if ch == 'l':
                        states.append((158, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 121:
                    if ch in ('1', '0'):
                        states.append((81, self.reader.save()))
                        continue
                    break
                case 122:
                    if ch == 'B':
                        states.append((159, self.reader.save()))
                        continue
                    break
                case 123:
                    if ch == 'b':
                        states.append((159, self.reader.save()))
                        continue
                    break
                case 124:
                    if ch == 'p':
                        states.append((160, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((161, self.reader.save()))
                        continue
                    if ch in ('2', '4', 'a', 'c', 'B', '1', '3', '9', '7', 'b', 'd', 'f', 'A', 'e', '5', 'C', 'D', 'F', '8', '6', 'E', '0'):
                        states.append((133, self.reader.save()))
                        continue
                    break
                case 125:
                    if ch == 'b':
                        states.append((162, self.reader.save()))
                        continue
                    break
                case 126:
                    if ch in ('+', '-'):
                        states.append((163, self.reader.save()))
                        continue
                    if ch in ('0', '1', '2', '3', '7', '6', '5', '4', '8', '9'):
                        states.append((164, self.reader.save()))
                        continue
                    break
                case 127:
                    if ch == 'l':
                        states.append((165, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 128:
                    if ch == 'B':
                        states.append((162, self.reader.save()))
                        continue
                    break
                case 129:
                    if ch in ('c', 'E', '3', 'b', 'D', '4', 'f', '7', 'd', 'F', '5', '0', 'A', '8', 'e', '6', '1', 'B', '9', '2', 'a', 'C'):
                        states.append((89, self.reader.save()))
                        continue
                    break
                case 130:
                    if ch == 'L':
                        states.append((165, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 131:
                    if ch in ('1', '0', '3', '2', '4', '5', '6', '9', '7', '8'):
                        states.append((166, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((167, self.reader.save()))
                        continue
                    break
                case 132:
                    if ch == 'l':
                        states.append((168, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((169, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((170, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((171, self.reader.save()))
                        continue
                    break
                case 133:
                    if ch == "'":
                        states.append((172, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((160, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((161, self.reader.save()))
                        continue
                    if ch in ('b', 'd', 'f', 'B', '1', '3', '7', '5', '9', 'D', 'F', 'C', 'c', 'e', 'A', '0', '2', '4', '6', '8', 'a', 'E'):
                        states.append((133, self.reader.save()))
                        continue
                    break
                case 134:
                    if ch == '\\':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('0', '4', '2', '6', '1', '3', '7', '5'):
                        states.append((55, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 135:
                    if ch == '\\':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('b', 'f', '3', 'D', '7', '0', 'c', 'A', 'E', '4', '8', 'a', 'C', '1', '9', 'd', 'B', 'F', '5', '2', 'e', '6'):
                        states.append((135, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 136:
                    if ch in ('5', 'd', '0', '2', 'a', 'c', '6', '8', '4', 'e', 'A', 'C', 'E', '1', '3', 'D', '9', 'F', 'b', 'B', 'f', '7'):
                        states.append((173, self.reader.save()))
                        continue
                    break
                case 137:
                    if ch in ('8', '1', '3', '9', 'b', '7', 'd', 'f', 'B', 'A', '5', 'D', 'F', '0', '4', 'E', 'a', 'c', 'e', '2', 'C', '6'):
                        states.append((174, self.reader.save()))
                        continue
                    break
                case 138:
                    if ch == '\\':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('0', '2', '6', '4', '1', '3', '5', '7'):
                        states.append((175, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 139:
                    if ch in ('1', 'B', '3', '9', 'b', '5', '7', 'd', 'f', 'D', 'F', '6', '0', '2', 'A', '4', 'a', 'c', '8', 'e', 'C', 'E'):
                        states.append((176, self.reader.save()))
                        continue
                    break
                case 140:
                    if ch in ('a', '2', '0', '4', '6', '8', 'c', 'e', 'A', 'C', 'E', '3', '1', '9', 'd', '5', '7', 'b', 'f', 'B', 'D', 'F'):
                        states.append((177, self.reader.save()))
                        continue
                    break
                case 141:
                    if ch in ('d', '1', '3', '9', '5', '7', 'f', 'B', 'b', 'D', 'F', '0', '2', '4', 'A', '6', '8', 'e', 'a', 'c', 'C', 'E'):
                        states.append((178, self.reader.save()))
                        continue
                    break
                case 142:
                    if ch in ('2', '0', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E', '1', '3', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'F'):
                        states.append((179, self.reader.save()))
                        continue
                    break
                case 143:
                    if ch in ('3', '1', '9', 'b', '7', '5', 'd', 'f', 'B', 'D', 'F', 'E', '0', '2', '4', 'a', '6', '8', 'c', 'e', 'A', 'C'):
                        states.append((180, self.reader.save()))
                        continue
                    break
                case 144:
                    if ch in ('2', '0', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'F', '1', '3', '9', 'b', '7', '5', 'd', 'f', 'B', 'D', 'E'):
                        states.append((181, self.reader.save()))
                        continue
                    break
                case 145:
                    if ch in ('1', '3', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'F', '2', '0', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E'):
                        states.append((182, self.reader.save()))
                        continue
                    break
                case 146:
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((107, self.reader.save()))
                        continue
                    break
                case 147:
                    if ch in ('1', '2', '3', '4', '0', '6', '7', '5', '8', '9'):
                        states.append((108, self.reader.save()))
                        continue
                    break
                case 148:
                    if ch in ('3', '0', '4', 'c', '8', 'A', 'E', '1', '9', 'd', '5', 'B', 'F', 'D', 'C', '2', 'a', 'e', '6', 'f', '7', 'b'):
                        states.append((183, self.reader.save()))
                        continue
                    break
                case 149:
                    if ch in ('5', 'd', '3', 'b', 'f', '1', '7', 'D', '2', 'e', '0', '4', 'c', 'A', '8', 'E', '9', 'B', 'F', 'a', 'C', '6'):
                        states.append((184, self.reader.save()))
                        continue
                    break
                case 150:
                    if ch == '\\':
                        states.append((80, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((41, self.reader.save()))
                        continue
                    if ch in ('0', '4', '1', '5', '2', '6', '3', '7'):
                        states.append((40, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 151:
                    if ch in ('3', 'b', 'f', '7', 'D', '0', '4', 'A', 'c', 'E', '8', '1', '9', 'd', 'B', 'F', '5', '2', 'a', 'e', '6', 'C'):
                        states.append((185, self.reader.save()))
                        continue
                    break
                case 152:
                    if ch in ('a', '2', 'e', '6', 'C', '3', '7', 'b', 'f', 'D', '4', '0', '8', 'c', 'E', 'A', '1', '9', '5', 'd', 'B', 'F'):
                        states.append((186, self.reader.save()))
                        continue
                    break
                case 153:
                    if ch == '\\':
                        states.append((80, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((41, self.reader.save()))
                        continue
                    if ch in ('0', 'A', '8', '1', '9', 'B', 'a', 'C', '2', '3', 'b', 'D', '4', 'c', 'E', 'd', 'F', '5', 'e', '6', 'f', '7'):
                        states.append((153, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 154:
                    if ch == 'b':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 155:
                    if ch == 'B':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 156:
                    if ch == 'L':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 157:
                    if ch == 'l':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 158:
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 159:
                    if ch in ('U', 'u'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 160:
                    if ch in ('+', '-'):
                        states.append((187, self.reader.save()))
                        continue
                    if ch in ('3', '0', '1', '2', '4', '5', '6', '7', '8', '9'):
                        states.append((188, self.reader.save()))
                        continue
                    break
                case 161:
                    if ch in ('6', '7', '8', '1', '2', '3', '4', '0', '9', '5'):
                        states.append((189, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((190, self.reader.save()))
                        continue
                    break
                case 162:
                    if ch in ('u', 'U'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 163:
                    if ch in ('0', '1', '2', '3', '7', '6', '5', '4', '8', '9'):
                        states.append((164, self.reader.save()))
                        continue
                    break
                case 164:
                    if ch == 'd':
                        states.append((191, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((192, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((193, self.reader.save()))
                        continue
                    if ch in ('3', '9', '2', '4', '0', '5', '6', '8', '1', '7'):
                        states.append((164, self.reader.save()))
                        continue
                    if ch in ('L', 'l', 'f', 'F'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 165:
                    if ch in ('U', 'u'):
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 166:
                    if ch == 'd':
                        states.append((191, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((192, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((194, self.reader.save()))
                        continue
                    if ch in ('0', '2', '4', '6', '8', '9', '1', '3', '5', '7'):
                        states.append((166, self.reader.save()))
                        continue
                    if ch in ('L', 'l', 'f', 'F'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 167:
                    if ch in ('1', '0', '3', '2', '4', '5', '6', '9', '7', '8'):
                        states.append((166, self.reader.save()))
                        continue
                    break
                case 168:
                    if ch == 'l':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 169:
                    if ch == 'L':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 170:
                    if ch == 'B':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 171:
                    if ch == 'b':
                        states.append((68, self.reader.save()))
                        continue
                    break
                case 172:
                    if ch in ('b', 'd', 'f', 'B', '1', '3', '7', '5', '9', 'D', 'F', 'c', 'e', 'A', '0', '2', '4', '6', '8', 'a', 'C', 'E'):
                        states.append((133, self.reader.save()))
                        continue
                    break
                case 173:
                    if ch in ('B', '3', '1', '9', '5', '7', 'b', 'd', 'f', 'D', 'F', 'A', '0', '2', '4', '6', '8', 'a', 'c', 'e', 'C', 'E'):
                        states.append((195, self.reader.save()))
                        continue
                    break
                case 174:
                    if ch in ('0', '2', '4', 'a', '6', '8', 'c', 'e', 'A', 'C', 'E', '3', '1', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'F'):
                        states.append((196, self.reader.save()))
                        continue
                    break
                case 175:
                    if ch == '\\':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('6', '1', '3', '5', '7', '0', '4', '2'):
                        states.append((55, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 176:
                    if ch == '\\':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('3', 'f', '7', 'b', 'D', '0', '4', 'A', '8', 'c', 'E', '1', '9', 'B', '5', 'd', 'F', '2', 'a', '6', 'e', 'C'):
                        states.append((176, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 177:
                    if ch in ('7', 'A', 'e', '2', '0', '6', '8', '4', 'a', 'c', 'C', 'E', 'B', '1', '3', '9', '5', 'b', 'd', 'f', 'D', 'F'):
                        states.append((197, self.reader.save()))
                        continue
                    break
                case 178:
                    if ch in ('8', 'd', 'f', 'B', '3', '5', '1', '7', '9', 'b', 'D', 'F', 'e', 'A', '0', '2', '6', '4', 'a', 'c', 'C', 'E'):
                        states.append((198, self.reader.save()))
                        continue
                    break
                case 179:
                    if ch in ('7', 'E', '0', '2', '4', 'a', 'c', 'e', 'A', '6', '8', 'C', '1', '9', 'b', 'd', 'f', 'B', 'D', 'F', '3', '5'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 180:
                    if ch in ('6', 'D', 'F', '1', '3', '9', 'b', 'd', 'f', 'B', '5', '2', '7', '0', 'a', 'c', 'e', 'A', 'C', 'E', '4', '8'):
                        states.append((199, self.reader.save()))
                        continue
                    break
                case 181:
                    if ch in ('2', '0', 'a', 'c', 'e', 'A', '4', '8', '6', 'C', 'E', 'b', 'd', 'f', 'B', 'D', 'F', '3', '1', '7', '9', '5'):
                        states.append((200, self.reader.save()))
                        continue
                    break
                case 182:
                    if ch in ('D', 'F', '3', '9', 'b', 'd', 'f', 'B', '5', '7', '4', '1', '2', 'c', 'e', 'A', 'C', 'E', '0', '6', 'a', '8'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 183:
                    if ch in ('1', '9', 'd', 'B', '2', 'a', 'e', 'F', '6', 'C', '5', '4', '0', 'b', 'f', 'D', '3', '7', 'E', 'A', '8', 'c'):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 184:
                    if ch in ('C', 'D', 'B', '9', 'd', '1', '5', 'F', '6', '7', '8', 'E', 'b', '4', '2', 'a', '3', '0', 'f', 'A', 'c', 'e'):
                        states.append((201, self.reader.save()))
                        continue
                    break
                case 185:
                    if ch in ('8', '1', '9', 'd', '5', 'B', 'F', '2', 'a', 'e', '6', 'C', 'f', '3', 'b', '7', 'D', '0', 'c', 'A', 'E', '4'):
                        states.append((202, self.reader.save()))
                        continue
                    break
                case 186:
                    if ch in ('7', '4', '0', 'c', 'A', '8', 'E', '1', '9', 'd', 'B', '5', 'F', '2', 'a', 'e', 'C', '6', 'b', 'f', 'D', '3'):
                        states.append((203, self.reader.save()))
                        continue
                    break
                case 187:
                    if ch in ('3', '0', '1', '2', '4', '5', '6', '7', '8', '9'):
                        states.append((188, self.reader.save()))
                        continue
                    break
                case 188:
                    if ch == 'd':
                        states.append((204, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((205, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((206, self.reader.save()))
                        continue
                    if ch in ('1', '3', '9', '5', '7', '0', '2', '4', '6', '8'):
                        states.append((188, self.reader.save()))
                        continue
                    if ch in ('f', 'F', 'L', 'l'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 189:
                    if ch == 'd':
                        states.append((204, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((205, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((207, self.reader.save()))
                        continue
                    if ch in ('f', 'F', 'L', 'l'):
                        states.append((73, self.reader.save()))
                        continue
                    if ch in ('1', '3', '9', '7', '5', '2', '4', '0', '6', '8'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 190:
                    if ch in ('5', '7', '8', '0', '2', '3', '4', '1', '9', '6'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 191:
                    if ch in ('d', 'l', 'f'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 192:
                    if ch in ('D', 'F', 'L'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 193:
                    if ch in ('0', '2', '3', '4', '9', '1', '5', '6', '7', '8'):
                        states.append((164, self.reader.save()))
                        continue
                    break
                case 194:
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((166, self.reader.save()))
                        continue
                    break
                case 195:
                    if ch in ('a', 'c', 'e', 'A', '6', '8', 'C', 'E', '2', '4', '1', '3', '9', 'b', 'd', 'f', '5', '7', 'B', 'D', 'F', '0'):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 196:
                    if ch in ('5', '9', 'b', 'd', '7', 'f', 'B', 'D', 'F', '3', '2', '4', '0', '6', 'a', 'c', 'e', '8', 'A', 'C', 'E', '1'):
                        states.append((208, self.reader.save()))
                        continue
                    break
                case 197:
                    if ch in ('1', '3', '9', 'b', 'd', '7', '5', 'f', 'B', 'D', 'F', '2', '0', '4', 'a', 'c', 'e', '6', '8', 'A', 'C', 'E'):
                        states.append((209, self.reader.save()))
                        continue
                    break
                case 198:
                    if ch in ('0', '2', '4', 'a', 'c', 'e', 'A', '6', '8', 'C', 'E', '1', '3', '5', '9', 'b', 'd', '7', 'f', 'B', 'D', 'F'):
                        states.append((210, self.reader.save()))
                        continue
                    break
                case 199:
                    if ch in ('1', '3', 'B', '5', '9', 'b', '7', 'd', 'f', 'D', 'F', '0', '2', '4', '6', 'a', 'c', 'e', '8', 'A', 'C', 'E'):
                        states.append((211, self.reader.save()))
                        continue
                    break
                case 200:
                    if ch in ('a', '0', '4', '2', '6', '8', 'c', 'e', 'A', 'C', 'E', 'b', '1', '3', '9', '5', '7', 'd', 'f', 'B', 'D', 'F'):
                        states.append((212, self.reader.save()))
                        continue
                    break
                case 201:
                    if ch in ('8', 'F', '7', '1', 'B', '9', 'd', '2', '5', '4', 'a', 'e', 'C', '6', 'c', '0', 'b', '3', 'f', 'D', 'A', 'E'):
                        states.append((213, self.reader.save()))
                        continue
                    break
                case 202:
                    if ch in ('2', 'a', 'e', 'C', '6', '3', 'b', 'f', 'D', '7', '0', '4', 'c', 'A', '8', 'E', 'F', '1', '9', 'd', '5', 'B'):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 203:
                    if ch in ('1', '9', 'd', 'B', '5', 'F', '2', 'a', 'e', 'C', '6', '3', 'b', 'f', 'D', '7', '0', '4', 'c', 'A', '8', 'E'):
                        states.append((214, self.reader.save()))
                        continue
                    break
                case 204:
                    if ch in ('l', 'f', 'd'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 205:
                    if ch in ('D', 'F', 'L'):
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 206:
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((188, self.reader.save()))
                        continue
                    break
                case 207:
                    if ch in ('1', '2', '3', '4', '0', '6', '7', '5', '8', '9'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 208:
                    if ch in ('D', 'F', '5', 'c', 'e', '2', '0', 'A', '6', '4', '7', '8', 'a', 'C', 'E', 'b', 'd', 'f', '1', '3', '9', 'B'):
                        states.append((215, self.reader.save()))
                        continue
                    break
                case 209:
                    if ch in ('6', 'e', 'A', 'C', '3', '1', '9', 'b', '5', '7', 'd', 'f', 'B', 'D', 'F', '0', '2', '4', 'E', 'c', 'a', '8'):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 210:
                    if ch in ('1', '3', '9', '0', '2', '4', 'a', '8', 'f', 'A', 'c', 'b', 'e', '6', 'C', 'E', '7', 'B', 'D', 'F', 'd', '5'):
                        states.append((216, self.reader.save()))
                        continue
                    break
                case 211:
                    if ch in ('a', '2', '0', '4', '6', '8', 'c', 'e', 'A', 'C', 'E', '3', '1', '9', 'd', '5', '7', 'f', 'b', 'B', 'D', 'F'):
                        states.append((217, self.reader.save()))
                        continue
                    break
                case 212:
                    if ch in ('f', '1', '3', '9', '5', '7', 'B', 'b', 'd', 'D', 'E', 'F', '0', '2', '4', 'A', '6', '8', 'a', 'c', 'e', 'C'):
                        states.append((218, self.reader.save()))
                        continue
                    break
                case 213:
                    if ch in ('2', 'a', '6', 'e', 'C', '0', 'D', 'f', '4', '3', 'b', '7', '1', 'B', 'E', '9', 'c', '8', 'A', 'F', '5', 'd'):
                        states.append((219, self.reader.save()))
                        continue
                    break
                case 214:
                    if ch in ('1', '9', '5', 'd', 'B', 'F', '2', 'a', 'e', 'C', '6', '3', 'b', 'f', 'D', '7', '0', '4', 'c', 'A', 'E', '8'):
                        states.append((220, self.reader.save()))
                        continue
                    break
                case 215:
                    if ch in ('b', 'd', 'f', 'B', '1', '7', '3', '5', '9', 'D', 'c', 'e', 'A', '0', '2', '4', '6', '8', 'a', 'C', 'F', 'E'):
                        states.append((221, self.reader.save()))
                        continue
                    break
                case 216:
                    if ch in ('a', 'c', 'e', 'A', '2', '0', '4', '6', '8', 'C', 'F', 'b', 'd', 'f', 'B', '3', '1', '5', '7', '9', 'D', 'E'):
                        states.append((222, self.reader.save()))
                        continue
                    break
                case 217:
                    if ch in ('7', 'A', 'e', '2', '0', '6', '8', '4', 'a', 'c', 'C', 'E', 'B', '1', '3', '9', '5', 'b', 'd', 'f', 'D', 'F'):
                        states.append((223, self.reader.save()))
                        continue
                    break
                case 218:
                    if ch in ('F', 'd', 'f', '1', '3', '7', '5', '9', 'b', 'B', 'D', '8', 'A', 'e', '2', '0', '6', '4', 'a', 'c', 'C', 'E'):
                        states.append((224, self.reader.save()))
                        continue
                    break
                case 219:
                    if ch in ('0', '4', 'c', '8', 'A', 'E', '1', '9', 'd', '5', 'B', 'F', '2', 'a', 'e', '6', 'C', '3', 'b', 'f', '7', 'D'):
                        states.append((225, self.reader.save()))
                        continue
                    break
                case 220:
                    if ch in ('3', 'b', 'f', '7', 'D', 'a', 'e', 'C', '4', '0', 'c', 'A', '8', 'E', '1', '9', 'd', 'B', '5', 'F', '2', '6'):
                        states.append((226, self.reader.save()))
                        continue
                    break
                case 221:
                    if ch in ('6', '8', 'C', 'E', '1', '0', 'B', 'f', 'd', '3', '5', '7', '9', 'b', 'D', 'F', '4', '2', 'c', 'a', 'e', 'A'):
                        states.append((227, self.reader.save()))
                        continue
                    break
                case 222:
                    if ch in ('b', '3', '1', '9', '7', '5', 'd', 'f', 'B', 'D', 'F', '0', '2', '4', 'a', 'e', '6', '8', 'A', 'c', 'C', 'E'):
                        states.append((228, self.reader.save()))
                        continue
                    break
                case 223:
                    if ch in ('1', '3', '9', 'b', 'd', '7', '5', 'f', 'B', 'D', 'F', '2', '0', '4', 'a', 'c', 'e', '6', '8', 'A', 'C', 'E'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 224:
                    if ch in ('2', '0', '4', '6', 'a', 'c', 'e', '8', 'A', 'C', 'E', '3', '1', '5', '7', '9', 'b', 'd', 'f', 'B', 'D', 'F'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 225:
                    if ch in ('1', '9', 'd', 'B', '5', 'F', '2', 'a', 'e', 'C', '6', 'b', 'f', 'D', '3', '7', '0', '4', 'c', 'A', '8', 'E'):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 226:
                    if ch in ('8', 'A', '0', '4', 'c', 'E', 'F', '9', 'd', 'B', '5', '1', '2', 'e', 'C', 'a', '6', 'D', 'b', 'f', '3', '7'):
                        states.append((229, self.reader.save()))
                        continue
                    break
                case 227:
                    if ch in ('a', '0', '4', '2', 'A', '6', '8', 'c', 'e', 'C', 'E', 'b', '1', '3', '9', '5', '7', 'd', 'f', 'B', 'D', 'F'):
                        states.append((55, self.reader.save()))
                        continue
                    break
                case 228:
                    if ch in ('F', '1', '3', '9', 'd', 'b', 'f', 'B', '7', '6', '8', '5', 'c', 'D', 'A', 'C', 'E', '2', '0', '4', 'a', 'e'):
                        states.append((230, self.reader.save()))
                        continue
                    break
                case 229:
                    if ch in ('5', 'd', 'B', 'F', 'e', '2', 'a', 'C', '6', 'f', '3', 'b', 'D', '7', '0', 'c', 'A', '4', 'E', '8', '1', '9'):
                        states.append((40, self.reader.save()))
                        continue
                    break
                case 230:
                    if ch in ('a', 'c', 'e', '0', '2', '6', '4', '8', 'A', 'C', 'E', 'b', 'd', '1', '3', '5', '7', '9', 'f', 'B', 'D', 'F'):
                        states.append((55, self.reader.save()))
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
                    return Token(TokenKind.INTCONST, location, text)
                case 4:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 5:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 6:
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
                    return Token(Token.punctuator[text], location, text)
                case 11:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 12:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 13:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 14:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
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
                    return Token(Token.punctuator[text], location, text)
                case 21:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 22:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 23:
                    self.reader.restore(back_index)
                    return Token(TokenKind.END, location, text)
                case 25:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 26:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 30:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 32:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 36:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 37:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 38:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 39:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 40:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 41:
                    self.reader.restore(back_index)
                    return Token(TokenKind.STRINGLITERAL, location, text)
                case 43:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 45:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 46:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 50:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 52:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 54:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 60:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 62:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 63:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 66:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 67:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 68:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 70:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 73:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 76:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 80:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 81:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 83:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 84:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 87:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 88:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 89:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 96:
                    self.reader.restore(back_index)
                    return self.error('未知的转义字符', location)
                case 97:
                    self.reader.restore(back_index)
                    return Token(TokenKind.CHARCONST, location, text)
                case 107:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 108:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 110:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 113:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 114:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 118:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 119:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 120:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 127:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 130:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 132:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 150:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 153:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 156:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 157:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 158:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 159:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 162:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 164:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 165:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 166:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 168:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 169:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 188:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 189:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
        self.reader.restore(start_index)
        return None