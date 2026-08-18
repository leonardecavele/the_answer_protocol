extern int add_42(int a, int b);

int main(void)
{
    if (add_42(0, 0) != 42 || add_42(8, -99) != 50)
        return 0;
    if (add_42(-42, 1234) != 0 || add_42(-100, 7) != -58)
        return 0;
    return SANDBOX_SUCCESS;
}
