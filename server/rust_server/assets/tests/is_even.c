extern int is_even(int n);

int main(void)
{
    if (is_even(0) != 1 || is_even(2) != 1 || is_even(-2) != 1)
        return 0;
    if (is_even(1) != 0 || is_even(-1) != 0)
        return 0;
    if (is_even(99) != 0 || is_even(-100) != 1)
        return 0;
    return SANDBOX_SUCCESS;
}
