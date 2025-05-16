"""生成所有"""

import os

os.chdir(os.path.join(os.path.dirname(__file__), ".."))

for command in (
    " ".join(
        ["python", "-m", "tools.LexBuilder", "lex/tokens.gram", "--class-name", "Lexer"]
    ),
    " ".join(
        [
            "python",
            "-m",
            "tools.ParseBuilder",
            "lex/pp_directive.gram",
            "--class-name",
            "PPDirectiveParser",
            "-o",
            "lex/gen_pp_directive_parser.py",
        ]
    ),
    " ".join(["python", "-m", "tools.ParseBuilder", "parse/c.gram"]),
    " ".join(
        [
            "python",
            "-m",
            "tools.ParseBuilder",
            "tools/ParseBuilder/builder.gram",
            "-o",
            "tools/ParseBuilder/gen_builder_parser.py",
        ]
    ),
):
    print(command)
    os.system(command)
