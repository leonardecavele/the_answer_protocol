extern int is_positive(int n);

int main(void)
{
    if (is_positive(1) != 1 || is_positive(999) != 1)
        return 0;
    if (is_positive(0) != 0 || is_positive(-1) != 0)
        return 0;
    if (is_positive(-999) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
