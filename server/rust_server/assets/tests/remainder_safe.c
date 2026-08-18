extern int remainder_safe(int a, int b);

int main(void)
{
    if (remainder_safe(7, 0) != 0 || remainder_safe(0, 3) != 0)
        return 0;
    if (remainder_safe(7, 3) != 1 || remainder_safe(8, 4) != 0)
        return 0;
    if (remainder_safe(-7, 3) != -1 || remainder_safe(7, -3) != 1)
        return 0;
    if (remainder_safe(-7, -3) != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
