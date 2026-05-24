use crate::codegen_test_template;

codegen_test_template!(
    var,
    "static int a;
extern int b;
thread_local static int c;
thread_local extern int d;
int main()
{
    static int a;
    extern int b;
    thread_local static int c;
    thread_local extern int d;
}
"
);

codegen_test_template!(
    compound_literal,
    "int main()
{
    (int){1};
    (static int){2};
    (thread_local static int){3};
}
"
);
