extern int is_negative(int n);

int main(void)
{
    if (is_negative(-1) != 1 || is_negative(-999) != 1)
        return 0;
    if (is_negative(0) != 0 || is_negative(1) != 0)
        return 0;
    if (is_negative(999) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
