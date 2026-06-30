use crate::codegen_test_template;

codegen_test_template!(
    switch,
    "int main()
{
    int a;
    switch (a)
    {
    case 0: a++;
    case 1: a++;
    case 2: a++;
    case 3: a++;
    default: a++;
    }
}
"
);
