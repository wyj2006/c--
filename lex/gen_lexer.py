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
                    if ch == '0':
                        states.append((1, self.reader.save()))
                        continue
                    if ch == '-':
                        states.append((4, self.reader.save()))
                        continue
                    if ch == '\\':
                        states.append((5, self.reader.save()))
                        continue
                    if ch == '&':
                        states.append((6, self.reader.save()))
                        continue
                    if ch == '^':
                        states.append((7, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((8, self.reader.save()))
                        continue
                    if ch == '|':
                        states.append((9, self.reader.save()))
                        continue
                    if ch == '#':
                        states.append((10, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((13, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((14, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((15, self.reader.save()))
                        continue
                    if ch == '<':
                        states.append((16, self.reader.save()))
                        continue
                    if ch == '*':
                        states.append((17, self.reader.save()))
                        continue
                    if ch == '>':
                        states.append((18, self.reader.save()))
                        continue
                    if ch == '+':
                        states.append((19, self.reader.save()))
                        continue
                    if ch == '!':
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
                    if ch == '=':
                        states.append((24, self.reader.save()))
                        continue
                    if ch == '':
                        states.append((25, self.reader.save()))
                        continue
                    if ch in ('q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'M', 'N', 'O', 'P', '_', 'Q', 'a', 'R', 'b', 'S', 'c', 'T', 'd', 'e', 'V', 'f', 'W', 'g', 'X', 'h', 'Y', 'i', 'Z', 'j', 'k', 'l', 'm', 'n', 'o', 'p'):
                        states.append((2, self.reader.save()))
                        continue
                    if ch in ('1', '2', '3', '4', '5', '6', '7', '8', '9'):
                        states.append((3, self.reader.save()))
                        continue
                    if ch in ('U', 'L'):
                        states.append((11, self.reader.save()))
                        continue
                    if ch in ('[', ']', '(', ')', '{', '}', '~', '?', ';', ','):
                        states.append((12, self.reader.save()))
                        continue
                    if self.other_identifier_start(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 1:
                    if ch == 'l':
                        states.append((27, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((31, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((32, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((33, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((36, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((38, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((39, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((26, self.reader.save()))
                        continue
                    if ch in ('X', 'x'):
                        states.append((28, self.reader.save()))
                        continue
                    if ch in ('0', '7', '5', '1', '6', '2', '3', '4'):
                        states.append((29, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((34, self.reader.save()))
                        continue
                    if ch in ('b', 'B'):
                        states.append((37, self.reader.save()))
                        continue
                    break
                case 2:
                    if ch == '\\':
                        states.append((40, self.reader.save()))
                        continue
                    if ch in ('G', '2', 'l', 'H', '3', 'm', 'I', '4', 'n', 'k', 'J', '5', 'o', '1', 'K', 'p', '6', 'L', '7', 'q', 'M', '8', 'r', 'N', '9', 's', 'O', 't', 'P', '_', 'u', 'Q', 'a', 'v', 'R', 'b', 'w', 'S', 'c', 'x', 'T', 'd', 'y', 'U', 'e', 'z', 'V', 'f', 'A', 'W', 'g', 'B', 'X', 'h', 'C', 'Y', 'i', 'D', 'Z', 'j', 'E', '0', 'F'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 3:
                    if ch == 'l':
                        states.append((41, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((42, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((33, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((43, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((36, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((45, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((46, self.reader.save()))
                        continue
                    if ch in ('8', '0', '7', '6', '9', '1', '2', '3', '4', '5'):
                        states.append((3, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((44, self.reader.save()))
                        continue
                    break
                case 4:
                    if ch in ('>', '-', '='):
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 5:
                    if ch == 'u':
                        states.append((47, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((48, self.reader.save()))
                        continue
                    break
                case 6:
                    if ch in ('=', '&'):
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 7:
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 8:
                    if ch == '\\':
                        states.append((40, self.reader.save()))
                        continue
                    if ch == '8':
                        states.append((11, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((13, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((14, self.reader.save()))
                        continue
                    if ch in ('G', '2', 'l', 'H', '3', 'm', 'I', '4', 'n', 'k', 'J', '5', 'o', '1', 'K', 'p', '6', 'L', '7', 'q', 'M', 'r', 'N', '9', 's', 'O', 't', 'P', '_', 'u', 'Q', 'a', 'v', 'R', 'b', 'w', 'S', 'c', 'x', 'T', 'd', 'y', 'U', 'e', 'z', 'V', 'f', 'A', 'W', 'g', 'B', 'X', 'h', 'C', 'Y', 'i', 'D', 'Z', 'j', 'E', '0', 'F'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 9:
                    if ch in ('=', '|'):
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 10:
                    if ch == '#':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 11:
                    if ch == '\\':
                        states.append((40, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((13, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((14, self.reader.save()))
                        continue
                    if ch in ('G', '2', 'l', 'H', '3', 'm', 'I', '4', 'n', 'J', '5', 'o', 'F', 'K', 'p', '6', 'L', '7', 'q', 'M', '8', 'r', 'N', '9', 's', 'O', 't', 'P', '_', 'u', 'Q', 'a', 'v', 'R', 'b', 'w', 'S', 'c', 'x', 'T', 'd', 'y', 'U', 'e', 'z', 'V', 'f', 'A', 'W', 'g', 'B', 'X', 'h', 'C', 'Y', 'i', 'D', 'Z', 'j', 'E', '0', 'k', '1'):
                        states.append((2, self.reader.save()))
                        continue
                    if self.other_identifier_continue(ch):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 12:
                    break
                case 13:
                    if ch == '\\':
                        states.append((49, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 14:
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
                case 15:
                    if ch == '.':
                        states.append((55, self.reader.save()))
                        continue
                    if ch in ('1', '9', '4', '7', '2', '5', '8', '3', '0', '6'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 16:
                    if ch == '<':
                        states.append((56, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 17:
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 18:
                    if ch == '>':
                        states.append((57, self.reader.save()))
                        continue
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 19:
                    if ch in ('=', '+'):
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 20:
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 21:
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 22:
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 23:
                    if ch == ':':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 24:
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 25:
                    break
                case 26:
                    if ch == 'E':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((58, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((33, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((36, self.reader.save()))
                        continue
                    if ch in ('8', '5', '3', '2', '7', '0', '9', '6', '4', '1'):
                        states.append((26, self.reader.save()))
                        continue
                    break
                case 27:
                    if ch == 'l':
                        states.append((60, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 28:
                    if ch == '.':
                        states.append((62, self.reader.save()))
                        continue
                    if ch in ('1', 'B', 'D', 'a', '2', 'C', 'E', 'b', '3', 'F', 'c', '0', '4', 'd', '5', 'e', '6', 'f', '7', 'A', '8', '9'):
                        states.append((61, self.reader.save()))
                        continue
                    break
                case 29:
                    if ch == 'l':
                        states.append((27, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((30, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((31, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((32, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((33, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((36, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((38, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((39, self.reader.save()))
                        continue
                    if ch in ('8', '9'):
                        states.append((26, self.reader.save()))
                        continue
                    if ch in ('0', '7', '5', '1', '6', '2', '3', '4'):
                        states.append((29, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((34, self.reader.save()))
                        continue
                    break
                case 30:
                    if ch in ('+', '-'):
                        states.append((63, self.reader.save()))
                        continue
                    if ch in ('4', '7', '1', '2', '5', '0', '8', '3', '6', '9'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 31:
                    if ch in ('8', '9'):
                        states.append((26, self.reader.save()))
                        continue
                    if ch in ('7', '0', '5', '1', '6', '2', '3', '4'):
                        states.append((29, self.reader.save()))
                        continue
                    break
                case 32:
                    if ch == 'L':
                        states.append((60, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 33:
                    if ch == 'D':
                        states.append((66, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((67, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((68, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((69, self.reader.save()))
                        continue
                    if ch in ('1', '7', '2', '8', '3', '9', '4', '5', '0', '6'):
                        states.append((54, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 34:
                    if ch == 'W':
                        states.append((70, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((71, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((72, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((73, self.reader.save()))
                        continue
                    break
                case 35:
                    break
                case 36:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((74, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((75, self.reader.save()))
                        continue
                    break
                case 37:
                    if ch in ('0', '1'):
                        states.append((76, self.reader.save()))
                        continue
                    break
                case 38:
                    if ch == 'b':
                        states.append((77, self.reader.save()))
                        continue
                    break
                case 39:
                    if ch == 'B':
                        states.append((77, self.reader.save()))
                        continue
                    break
                case 40:
                    if ch == 'u':
                        states.append((78, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((79, self.reader.save()))
                        continue
                    break
                case 41:
                    if ch == 'l':
                        states.append((80, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 42:
                    if ch in ('8', '7', '0', '6', '9', '1', '2', '3', '4', '5'):
                        states.append((3, self.reader.save()))
                        continue
                    break
                case 43:
                    if ch == 'L':
                        states.append((80, self.reader.save()))
                        continue
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 44:
                    if ch == 'L':
                        states.append((81, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((82, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((83, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((84, self.reader.save()))
                        continue
                    break
                case 45:
                    if ch == 'b':
                        states.append((85, self.reader.save()))
                        continue
                    break
                case 46:
                    if ch == 'B':
                        states.append((85, self.reader.save()))
                        continue
                    break
                case 47:
                    if ch in ('2', '1', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', 'c', '7'):
                        states.append((86, self.reader.save()))
                        continue
                    break
                case 48:
                    if ch in ('5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a'):
                        states.append((87, self.reader.save()))
                        continue
                    break
                case 49:
                    if ch == 'u':
                        states.append((89, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((90, self.reader.save()))
                        continue
                    if ch == 'U':
                        states.append((91, self.reader.save()))
                        continue
                    if ch in ('4', '6', '1', '5', '7', '2', '3', '0'):
                        states.append((88, self.reader.save()))
                        continue
                    if ch in ('f', 'n', "'", 'r', '"', 't', '?', 'v', '\\', 'a', 'b'):
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
                    if ch == 'U':
                        states.append((95, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((96, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((97, self.reader.save()))
                        continue
                    if ch in ('5', '3', '0', '6', '4', '1', '7', '2'):
                        states.append((94, self.reader.save()))
                        continue
                    if ch in ('b', "'", 'f', '"', 'n', 'a', '?', 'r', '\\', 't', 'v'):
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
                    if ch == "'":
                        states.append((99, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((66, self.reader.save()))
                        continue
                    if ch == 'e':
                        states.append((67, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((68, self.reader.save()))
                        continue
                    if ch == 'E':
                        states.append((69, self.reader.save()))
                        continue
                    if ch in ('4', '5', '0', '6', '1', '7', '2', '8', '3', '9'):
                        states.append((54, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 55:
                    if ch == '.':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 56:
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 57:
                    if ch == '=':
                        states.append((12, self.reader.save()))
                        continue
                    break
                case 58:
                    if ch in ('8', '5', '3', '2', '7', '0', '9', '6', '4', '1'):
                        states.append((26, self.reader.save()))
                        continue
                    break
                case 59:
                    break
                case 60:
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 61:
                    if ch == "'":
                        states.append((100, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((101, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((102, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((104, self.reader.save()))
                        continue
                    if ch == '.':
                        states.append((105, self.reader.save()))
                        continue
                    if ch == 'P':
                        states.append((106, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((107, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((108, self.reader.save()))
                        continue
                    if ch in ('d', '6', 'D', 'f', 'e', '7', 'E', 'A', '8', 'F', 'B', '0', '9', 'C', '1', 'a', '2', 'b', '3', 'c', '4', '5'):
                        states.append((61, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((103, self.reader.save()))
                        continue
                    break
                case 62:
                    if ch in ('d', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'C', 'b', '6', '1', 'B', 'c', '7', '2'):
                        states.append((109, self.reader.save()))
                        continue
                    break
                case 63:
                    if ch in ('4', '7', '2', '9', '5', '0', '8', '3', '6', '1'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 64:
                    if ch == 'd':
                        states.append((110, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((111, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((112, self.reader.save()))
                        continue
                    if ch in ('7', '2', '8', '3', '9', '4', '5', '0', '6', '1'):
                        states.append((64, self.reader.save()))
                        continue
                    if ch in ('L', 'F', 'f', 'l'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 65:
                    break
                case 66:
                    if ch in ('F', 'D', 'L'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 67:
                    if ch in ('1', '6', '9', '3', '4', '7', '2', '5', '0', '8'):
                        states.append((113, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((114, self.reader.save()))
                        continue
                    break
                case 68:
                    if ch in ('l', 'f', 'd'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 69:
                    if ch in ('1', '9', '4', '7', '2', '5', '8', '3', '0', '6'):
                        states.append((115, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((116, self.reader.save()))
                        continue
                    break
                case 70:
                    if ch == 'B':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 71:
                    if ch == 'L':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 72:
                    if ch == 'l':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 73:
                    if ch == 'b':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 74:
                    if ch == "'":
                        states.append((117, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((110, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((111, self.reader.save()))
                        continue
                    if ch in ('4', '5', '0', '6', '1', '7', '2', '8', '3', '9'):
                        states.append((74, self.reader.save()))
                        continue
                    if ch in ('L', 'F', 'f', 'l'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 75:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 76:
                    if ch == "'":
                        states.append((118, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((120, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((121, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((122, self.reader.save()))
                        continue
                    if ch == 'l':
                        states.append((123, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((119, self.reader.save()))
                        continue
                    if ch in ('0', '1'):
                        states.append((76, self.reader.save()))
                        continue
                    break
                case 77:
                    if ch in ('u', 'U'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 78:
                    if ch in ('d', '8', '3', 'D', 'e', '9', 'C', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2'):
                        states.append((124, self.reader.save()))
                        continue
                    break
                case 79:
                    if ch in ('0', 'A', 'b', '6', '1', 'B', 'F', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5'):
                        states.append((125, self.reader.save()))
                        continue
                    break
                case 80:
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 81:
                    if ch == 'L':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 82:
                    if ch == 'l':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 83:
                    if ch == 'b':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 84:
                    if ch == 'B':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 85:
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 86:
                    if ch in ('6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', 'A', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'b'):
                        states.append((126, self.reader.save()))
                        continue
                    break
                case 87:
                    if ch in ('4', 'E', 'D', 'a', 'f', '5', '0', 'F', 'b', 'A', '6', '1', 'c', 'B', '7', '2', 'd', 'C', '8', '3', 'e', '9'):
                        states.append((127, self.reader.save()))
                        continue
                    break
                case 88:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('3', '7', '4', '2', '5', '0', '6', '1'):
                        states.append((128, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 89:
                    if ch in ('6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', 'F', '4', 'E', 'f', 'a', '0', '5', 'A', 'b'):
                        states.append((129, self.reader.save()))
                        continue
                    break
                case 90:
                    if ch in ('3', 'D', '9', 'e', '4', 'E', 'a', 'f', '5', '0', 'F', 'b', 'A', '6', 'C', '1', 'c', 'B', '7', '2', 'd', '8'):
                        states.append((130, self.reader.save()))
                        continue
                    break
                case 91:
                    if ch in ('9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'D', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'e'):
                        states.append((131, self.reader.save()))
                        continue
                    break
                case 92:
                    if ch == 'U':
                        states.append((132, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((134, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((135, self.reader.save()))
                        continue
                    if ch in ('3', '5', '0', '4', '6', '1', '7', '2'):
                        states.append((133, self.reader.save()))
                        continue
                    if ch in ('b', 'f', 'n', "'", 'r', '"', 't', '?', 'v', '\\', 'a'):
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
                    if ch in ('7', '2', '4', '3', '5', '0', '6', '1'):
                        states.append((136, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 95:
                    if ch in ('8', 'a', 'D', 'F', '0', '9', 'b', 'E', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C'):
                        states.append((137, self.reader.save()))
                        continue
                    break
                case 96:
                    if ch in ('5', '7', 'A', 'C', '6', '8', 'B', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f'):
                        states.append((138, self.reader.save()))
                        continue
                    break
                case 97:
                    if ch in ('e', '2', 'd', '4', 'f', '3', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', '0', 'F', 'b', '1', 'c'):
                        states.append((139, self.reader.save()))
                        continue
                    break
                case 98:
                    if ch == 'U':
                        states.append((141, self.reader.save()))
                        continue
                    if ch == 'u':
                        states.append((142, self.reader.save()))
                        continue
                    if ch == 'x':
                        states.append((143, self.reader.save()))
                        continue
                    if ch in ('4', '2', '5', '3', '0', '6', '1', '7'):
                        states.append((140, self.reader.save()))
                        continue
                    if ch in ('a', 'b', "'", 'f', '"', 'n', '?', 'r', '\\', 't', 'v'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 99:
                    if ch in ('4', '7', '2', '9', '5', '0', '8', '3', '6', '1'):
                        states.append((54, self.reader.save()))
                        continue
                    break
                case 100:
                    if ch in ('d', '6', 'D', 'f', 'e', '7', 'E', 'A', '8', 'F', 'B', '0', '9', 'C', '1', 'a', '2', 'b', '3', 'c', '4', '5'):
                        states.append((61, self.reader.save()))
                        continue
                    break
                case 101:
                    if ch == 'l':
                        states.append((144, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 102:
                    if ch == 'L':
                        states.append((144, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 103:
                    if ch == 'l':
                        states.append((145, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((146, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((147, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((148, self.reader.save()))
                        continue
                    break
                case 104:
                    if ch in ('7', '1', '2', '5', '0', '8', '3', '6', '4', '9'):
                        states.append((149, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((150, self.reader.save()))
                        continue
                    break
                case 105:
                    if ch == 'P':
                        states.append((151, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((152, self.reader.save()))
                        continue
                    if ch in ('d', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '2', '0', 'A', 'b', '6', '1', 'B', 'c', '7', 'C'):
                        states.append((109, self.reader.save()))
                        continue
                    break
                case 106:
                    if ch in ('2', '5', '0', '8', '3', '4', '6', '1', '9', '7'):
                        states.append((153, self.reader.save()))
                        continue
                    if ch in ('-', '+'):
                        states.append((154, self.reader.save()))
                        continue
                    break
                case 107:
                    if ch == 'b':
                        states.append((155, self.reader.save()))
                        continue
                    break
                case 108:
                    if ch == 'B':
                        states.append((155, self.reader.save()))
                        continue
                    break
                case 109:
                    if ch == 'P':
                        states.append((151, self.reader.save()))
                        continue
                    if ch == 'p':
                        states.append((152, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((156, self.reader.save()))
                        continue
                    if ch in ('4', 'f', 'E', 'a', '5', '0', 'A', 'F', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9'):
                        states.append((109, self.reader.save()))
                        continue
                    break
                case 110:
                    if ch in ('f', 'd', 'l'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 111:
                    if ch in ('D', 'L', 'F'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 112:
                    if ch in ('7', '2', '5', '0', '8', '3', '6', '1', '9', '4'):
                        states.append((64, self.reader.save()))
                        continue
                    break
                case 113:
                    if ch == 'D':
                        states.append((66, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((68, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((157, self.reader.save()))
                        continue
                    if ch in ('1', '7', '2', '8', '3', '9', '4', '5', '0', '6'):
                        states.append((113, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 114:
                    if ch in ('1', '6', '9', '8', '4', '7', '2', '0', '5', '3'):
                        states.append((113, self.reader.save()))
                        continue
                    break
                case 115:
                    if ch == "'":
                        states.append((158, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((66, self.reader.save()))
                        continue
                    if ch == 'd':
                        states.append((68, self.reader.save()))
                        continue
                    if ch in ('4', '5', '0', '6', '1', '7', '2', '8', '3', '9'):
                        states.append((115, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 116:
                    if ch in ('8', '1', '9', '4', '7', '2', '5', '0', '3', '6'):
                        states.append((115, self.reader.save()))
                        continue
                    break
                case 117:
                    if ch in ('4', '7', '9', '2', '5', '0', '8', '3', '6', '1'):
                        states.append((74, self.reader.save()))
                        continue
                    break
                case 118:
                    if ch in ('1', '0'):
                        states.append((76, self.reader.save()))
                        continue
                    break
                case 119:
                    if ch == 'l':
                        states.append((159, self.reader.save()))
                        continue
                    if ch == 'L':
                        states.append((160, self.reader.save()))
                        continue
                    if ch == 'w':
                        states.append((161, self.reader.save()))
                        continue
                    if ch == 'W':
                        states.append((162, self.reader.save()))
                        continue
                    break
                case 120:
                    if ch == 'b':
                        states.append((163, self.reader.save()))
                        continue
                    break
                case 121:
                    if ch == 'L':
                        states.append((164, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 122:
                    if ch == 'B':
                        states.append((163, self.reader.save()))
                        continue
                    break
                case 123:
                    if ch == 'l':
                        states.append((164, self.reader.save()))
                        continue
                    if ch in ('u', 'U'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 124:
                    if ch in ('1', 'c', 'B', '7', '2', 'd', 'C', '8', '3', 'e', 'D', '9', '4', 'f', 'E', 'a', '5', '0', 'A', 'F', 'b', '6'):
                        states.append((165, self.reader.save()))
                        continue
                    break
                case 125:
                    if ch in ('E', 'f', '5', 'a', '0', 'F', 'A', 'b', '6', '1', 'B', '7', 'c', '2', 'e', 'C', '8', 'd', '3', 'D', '9', '4'):
                        states.append((166, self.reader.save()))
                        continue
                    break
                case 126:
                    if ch in ('a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'E', 'e', '9', '4', 'f'):
                        states.append((167, self.reader.save()))
                        continue
                    break
                case 127:
                    if ch in ('d', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'C', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2'):
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
                    if ch in ('5', '0', '6', '4', '1', '7', '2', '3'):
                        states.append((50, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 129:
                    if ch in ('a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f'):
                        states.append((169, self.reader.save()))
                        continue
                    break
                case 130:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F'):
                        states.append((130, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 131:
                    if ch in ('8', '3', 'e', 'D', '9', '4', 'f', 'E', 'a', '5', '0', 'A', 'F', 'b', 'C', '6', '1', 'B', 'c', '7', '2', 'd'):
                        states.append((170, self.reader.save()))
                        continue
                    break
                case 132:
                    if ch in ('3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', 'A', '0', 'b', 'C', '6', '1', 'B', 'c', '7', '2', 'd', '8'):
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
                    if ch in ('7', '2', '3', '6', '4', '5', '1', '0'):
                        states.append((172, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 134:
                    if ch in ('5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a'):
                        states.append((173, self.reader.save()))
                        continue
                    break
                case 135:
                    if ch in ('2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7'):
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
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7'):
                        states.append((52, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 137:
                    if ch in ('D', '7', 'C', '9', 'E', '8', 'a', '0', 'F', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B'):
                        states.append((175, self.reader.save()))
                        continue
                    break
                case 138:
                    if ch in ('A', '4', '6', 'f', 'B', '5', '7', 'C', '8', 'D', '9', 'E', 'a', '0', 'F', 'b', '1', 'c', '2', 'd', '3', 'e'):
                        states.append((176, self.reader.save()))
                        continue
                    break
                case 139:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('a', '1', 'b', '2', 'c', '3', 'd', '4', 'e', '5', 'f', '6', 'A', '7', 'B', '8', 'C', '9', 'D', 'E', 'F', '0'):
                        states.append((139, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 140:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('6', '2', '1', '3', '7', '4', '5', '0'):
                        states.append((177, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 141:
                    if ch in ('9', 'C', 'E', '8', 'a', 'D', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7'):
                        states.append((178, self.reader.save()))
                        continue
                    break
                case 142:
                    if ch in ('f', '6', 'B', '5', 'A', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4'):
                        states.append((179, self.reader.save()))
                        continue
                    break
                case 143:
                    if ch in ('1', '3', 'c', 'e', '2', '4', 'd', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b'):
                        states.append((180, self.reader.save()))
                        continue
                    break
                case 144:
                    if ch in ('u', 'U'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 145:
                    if ch == 'l':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 146:
                    if ch == 'L':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 147:
                    if ch == 'b':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 148:
                    if ch == 'B':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 149:
                    if ch == 'd':
                        states.append((181, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((182, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((183, self.reader.save()))
                        continue
                    if ch in ('5', '0', '6', '1', '7', '2', '8', '3', '9', '4'):
                        states.append((149, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 150:
                    if ch in ('7', '2', '9', '5', '0', '8', '3', '6', '1', '4'):
                        states.append((149, self.reader.save()))
                        continue
                    break
                case 151:
                    if ch in ('0', '8', '3', '2', '6', '1', '9', '4', '7', '5'):
                        states.append((184, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((185, self.reader.save()))
                        continue
                    break
                case 152:
                    if ch in ('0', '8', '3', '6', '1', '9', '4', '5', '7', '2'):
                        states.append((186, self.reader.save()))
                        continue
                    if ch in ('+', '-'):
                        states.append((187, self.reader.save()))
                        continue
                    break
                case 153:
                    if ch == 'd':
                        states.append((181, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((182, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((188, self.reader.save()))
                        continue
                    if ch in ('8', '3', '9', '4', '5', '0', '6', '1', '7', '2'):
                        states.append((153, self.reader.save()))
                        continue
                    if ch in ('f', 'l', 'F', 'L'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 154:
                    if ch in ('2', '5', '0', '8', '3', '6', '1', '9', '4', '7'):
                        states.append((153, self.reader.save()))
                        continue
                    break
                case 155:
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 156:
                    if ch in ('4', 'f', 'E', 'a', '5', '0', 'A', 'F', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9'):
                        states.append((109, self.reader.save()))
                        continue
                    break
                case 157:
                    if ch in ('1', '9', '4', '7', '2', '5', '0', '8', '3', '6'):
                        states.append((113, self.reader.save()))
                        continue
                    break
                case 158:
                    if ch in ('4', '7', '2', '9', '5', '0', '8', '3', '6', '1'):
                        states.append((115, self.reader.save()))
                        continue
                    break
                case 159:
                    if ch == 'l':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 160:
                    if ch == 'L':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 161:
                    if ch == 'b':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 162:
                    if ch == 'B':
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 163:
                    if ch in ('U', 'u'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 164:
                    if ch in ('u', 'U'):
                        states.append((59, self.reader.save()))
                        continue
                    break
                case 165:
                    if ch in ('5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', 'E', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'f', 'a'):
                        states.append((189, self.reader.save()))
                        continue
                    break
                case 166:
                    if ch in ('8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'C', '1', 'B', 'c', '7', '2', 'd'):
                        states.append((190, self.reader.save()))
                        continue
                    break
                case 167:
                    if ch in ('e', 'D', '9', '4', 'E', 'f', 'a', '5', 'F', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 168:
                    if ch in ('c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', '1', 'a', '5', 'F', '0', 'A', 'b', '6', 'B'):
                        states.append((191, self.reader.save()))
                        continue
                    break
                case 169:
                    if ch in ('e', '9', '3', '4', 'E', 'f', 'a', '0', '5', 'F', 'A', 'b', '1', '6', 'B', 'c', '7', '2', 'C', 'd', '8', 'D'):
                        states.append((192, self.reader.save()))
                        continue
                    break
                case 170:
                    if ch in ('1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6'):
                        states.append((193, self.reader.save()))
                        continue
                    break
                case 171:
                    if ch in ('7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', '1', 'b', '6', 'B', 'c'):
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
                    if ch in ('4', '5', '0', '6', '1', '3', '7', '2'):
                        states.append((50, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 173:
                    if ch in ('4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', 'D', '2', 'C', 'd', '8', '3', 'e', '9'):
                        states.append((195, self.reader.save()))
                        continue
                    break
                case 174:
                    if ch == '\\':
                        states.append((92, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((93, self.reader.save()))
                        continue
                    if ch in ('a', '0', 'F', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E'):
                        states.append((174, self.reader.save()))
                        continue
                    if self.other_c_char(ch):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 175:
                    if ch in ('7', 'C', '6', 'B', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A'):
                        states.append((196, self.reader.save()))
                        continue
                    break
                case 176:
                    if ch in ('d', '4', 'f', '3', 'e', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2'):
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
                    if ch in ('0', '1', '2', '3', '4', '5', '6', '7'):
                        states.append((52, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 178:
                    if ch in ('6', '8', 'B', 'D', '7', '9', 'C', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A'):
                        states.append((198, self.reader.save()))
                        continue
                    break
                case 179:
                    if ch in ('3', '5', 'e', 'A', '4', '6', 'f', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd'):
                        states.append((199, self.reader.save()))
                        continue
                    break
                case 180:
                    if ch == '\\':
                        states.append((98, self.reader.save()))
                        continue
                    if ch == '"':
                        states.append((53, self.reader.save()))
                        continue
                    if ch in ('F', '9', '0', 'a', '1', 'b', '2', 'c', '3', 'd', '4', 'e', '5', 'f', '6', 'A', '7', 'B', '8', 'C', 'D', 'E'):
                        states.append((180, self.reader.save()))
                        continue
                    if self.other_s_char(ch):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 181:
                    if ch in ('d', 'l', 'f'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 182:
                    if ch in ('L', 'F', 'D'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 183:
                    if ch in ('2', '5', '0', '8', '3', '6', '1', '9', '4', '7'):
                        states.append((149, self.reader.save()))
                        continue
                    break
                case 184:
                    if ch == 'd':
                        states.append((200, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((201, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((202, self.reader.save()))
                        continue
                    if ch in ('6', '1', '7', '2', '8', '3', '9', '4', '5', '0'):
                        states.append((184, self.reader.save()))
                        continue
                    if ch in ('F', 'L', 'l', 'f'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 185:
                    if ch in ('0', '8', '3', '6', '1', '9', '4', '7', '2', '5'):
                        states.append((184, self.reader.save()))
                        continue
                    break
                case 186:
                    if ch == 'd':
                        states.append((200, self.reader.save()))
                        continue
                    if ch == 'D':
                        states.append((201, self.reader.save()))
                        continue
                    if ch == "'":
                        states.append((203, self.reader.save()))
                        continue
                    if ch in ('3', '9', '4', '5', '0', '6', '1', '7', '2', '8'):
                        states.append((186, self.reader.save()))
                        continue
                    if ch in ('F', 'L', 'f', 'l'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 187:
                    if ch in ('0', '8', '3', '6', '1', '9', '4', '7', '2', '5'):
                        states.append((186, self.reader.save()))
                        continue
                    break
                case 188:
                    if ch in ('5', '0', '8', '3', '6', '1', '9', '4', '7', '2'):
                        states.append((153, self.reader.save()))
                        continue
                    break
                case 189:
                    if ch in ('9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'D', 'd', '8', '3', 'e'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 190:
                    if ch in ('7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c'):
                        states.append((204, self.reader.save()))
                        continue
                    break
                case 191:
                    if ch in ('f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', 'E', '3', 'D', 'e', '9', '4'):
                        states.append((205, self.reader.save()))
                        continue
                    break
                case 192:
                    if ch in ('2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 193:
                    if ch in ('0', 'A', 'b', '6', 'F', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5'):
                        states.append((206, self.reader.save()))
                        continue
                    break
                case 194:
                    if ch in ('0', '6', '1', 'c', 'B', '7', '2', 'd', 'C', '8', '3', 'e', 'D', '9', 'F', '4', 'f', 'E', 'a', '5', 'A', 'b'):
                        states.append((207, self.reader.save()))
                        continue
                    break
                case 195:
                    if ch in ('8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd'):
                        states.append((208, self.reader.save()))
                        continue
                    break
                case 196:
                    if ch in ('6', 'f', 'B', '5', '7', 'A', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e', '4'):
                        states.append((209, self.reader.save()))
                        continue
                    break
                case 197:
                    if ch in ('c', '3', 'e', '2', 'd', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 198:
                    if ch in ('B', '5', 'A', '7', 'C', '6', '8', 'D', '9', 'E', 'a', '0', 'F', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f'):
                        states.append((210, self.reader.save()))
                        continue
                    break
                case 199:
                    if ch in ('e', '2', '4', 'd', 'f', '3', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', '0', 'F', 'b', '1', 'c'):
                        states.append((211, self.reader.save()))
                        continue
                    break
                case 200:
                    if ch in ('f', 'd', 'l'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 201:
                    if ch in ('D', 'L', 'F'):
                        states.append((65, self.reader.save()))
                        continue
                    break
                case 202:
                    if ch in ('3', '8', '6', '1', '9', '4', '7', '2', '5', '0'):
                        states.append((184, self.reader.save()))
                        continue
                    break
                case 203:
                    if ch in ('3', '6', '1', '9', '4', '7', '2', '8', '5', '0'):
                        states.append((186, self.reader.save()))
                        continue
                    break
                case 204:
                    if ch in ('a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f'):
                        states.append((212, self.reader.save()))
                        continue
                    break
                case 205:
                    if ch in ('D', 'e', 'd', '4', '9', 'E', 'f', 'a', '5', '0', 'F', 'A', '6', 'b', '1', 'B', '7', 'c', '2', 'C', '8', '3'):
                        states.append((213, self.reader.save()))
                        continue
                    break
                case 206:
                    if ch in ('3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', 'A', '0', 'b', '6', '1', 'B', 'c', '7', '2', 'C', 'd', '8'):
                        states.append((214, self.reader.save()))
                        continue
                    break
                case 207:
                    if ch in ('f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', 'E', '4'):
                        states.append((215, self.reader.save()))
                        continue
                    break
                case 208:
                    if ch in ('c', '7', '1', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', '0', 'F', 'A', 'b', '6', 'B'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 209:
                    if ch in ('2', '4', 'd', 'f', '3', '5', 'e', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c'):
                        states.append((216, self.reader.save()))
                        continue
                    break
                case 210:
                    if ch in ('5', 'A', '4', 'f', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1', 'c', '2', 'd', '3', 'e'):
                        states.append((217, self.reader.save()))
                        continue
                    break
                case 211:
                    if ch in ('b', '2', 'd', '1', 'c', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 212:
                    if ch in ('e', '9', '3', '4', 'E', 'f', 'a', '0', '5', 'F', 'A', 'b', '1', '6', 'B', 'c', '7', '2', 'C', 'd', '8', 'D'):
                        states.append((218, self.reader.save()))
                        continue
                    break
                case 213:
                    if ch in ('7', '1', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', 'B', 'c'):
                        states.append((219, self.reader.save()))
                        continue
                    break
                case 214:
                    if ch in ('2', 'C', 'B', '8', 'd', '3', 'D', '9', 'e', '4', 'E', 'a', 'f', '5', '0', 'F', 'b', 'A', '6', '1', 'c', '7'):
                        states.append((220, self.reader.save()))
                        continue
                    break
                case 215:
                    if ch in ('d', '8', '3', 'D', 'C', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', '2'):
                        states.append((221, self.reader.save()))
                        continue
                    break
                case 216:
                    if ch in ('d', '1', 'c', '3', 'e', '2', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', '0', 'F', 'b'):
                        states.append((222, self.reader.save()))
                        continue
                    break
                case 217:
                    if ch in ('3', 'c', 'e', '2', '4', 'd', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F', '0', 'b', '1'):
                        states.append((223, self.reader.save()))
                        continue
                    break
                case 218:
                    if ch in ('2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7'):
                        states.append((224, self.reader.save()))
                        continue
                    break
                case 219:
                    if ch in ('6', '1', 'B', 'F', 'c', '7', '2', 'C', '0', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'A', 'b'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 220:
                    if ch in ('b', '6', '1', 'B', 'F', 'c', '7', '2', '0', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'A'):
                        states.append((225, self.reader.save()))
                        continue
                    break
                case 221:
                    if ch in ('1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6'):
                        states.append((226, self.reader.save()))
                        continue
                    break
                case 222:
                    if ch in ('1', 'F', 'c', '0', 'b', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a'):
                        states.append((227, self.reader.save()))
                        continue
                    break
                case 223:
                    if ch in ('0', '2', 'b', 'd', '1', '3', 'c', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9', 'E', 'a', 'F'):
                        states.append((228, self.reader.save()))
                        continue
                    break
                case 224:
                    if ch in ('1', 'B', 'c', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '0', '4', 'E', 'f', 'a', '5', 'F', 'A', 'b', '6'):
                        states.append((2, self.reader.save()))
                        continue
                    break
                case 225:
                    if ch in ('a', '5', 'F', '0', 'A', 'b', '6', 'B', '1', 'c', 'E', '7', '2', 'C', 'd', '8', '3', 'D', 'e', '9', '4', 'f'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 226:
                    if ch in ('0', 'F', 'A', '6', 'b', '1', 'B', 'c', '7', '2', 'C', '8', 'd', '3', 'f', 'D', '9', 'e', '4', 'E', 'a', '5'):
                        states.append((229, self.reader.save()))
                        continue
                    break
                case 227:
                    if ch in ('0', '9', 'b', 'E', '1', 'a', 'c', 'F', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D'):
                        states.append((52, self.reader.save()))
                        continue
                    break
                case 228:
                    if ch in ('E', 'b', 'a', '1', 'F', 'c', '0', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8', 'D', '9'):
                        states.append((230, self.reader.save()))
                        continue
                    break
                case 229:
                    if ch in ('9', '4', 'E', 'f', 'a', '5', 'F', '0', 'A', 'b', '6', '1', 'B', 'c', '7', 'D', '2', 'C', 'd', '8', '3', 'e'):
                        states.append((50, self.reader.save()))
                        continue
                    break
                case 230:
                    if ch in ('D', 'a', 'F', '9', '0', 'E', 'b', '1', 'c', '2', 'd', '3', 'e', '4', 'f', '5', 'A', '6', 'B', '7', 'C', '8'):
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
                    return Token(TokenKind.INTCONST, location, text)
                case 2:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 3:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 4:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 6:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 7:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 8:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 9:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 10:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 11:
                    self.reader.restore(back_index)
                    return Token(TokenKind.IDENTIFIER if text not in Token.keywords else Token.keywords[text], location, text)
                case 12:
                    self.reader.restore(back_index)
                    return Token(Token.punctuator[text], location, text)
                case 14:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
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
                case 27:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 29:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 32:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 33:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 34:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 41:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 43:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 44:
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
                    return Token(TokenKind.FLOATCONST, location, text)
                case 65:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 71:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 72:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 74:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 76:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 77:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 80:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 81:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 82:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 85:
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
                case 101:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 102:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 103:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 113:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 115:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 119:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 121:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 123:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 136:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 139:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 140:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 144:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 145:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 146:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 149:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 153:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 155:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 159:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 160:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 163:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 164:
                    self.reader.restore(back_index)
                    return Token(TokenKind.INTCONST, location, text)
                case 177:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 180:
                    self.reader.restore(back_index)
                    return self.error('字符串未终止', location)
                case 184:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
                case 186:
                    self.reader.restore(back_index)
                    return Token(TokenKind.FLOATCONST, location, text)
        self.reader.restore(start_index)
        return None