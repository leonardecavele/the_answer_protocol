extern int negate(int n);

int main(void)
{
    if (negate(0) != 0 || negate(1) != -1)
        return 0;
    if (negate(-1) != 1 || negate(42) != -42)
        return 0;
    if (negate(-999) != 999)
        return 0;
    return SANDBOX_SUCCESS;
}
