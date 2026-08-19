extern int sum_digits(int n);

int main(void)
{
    if (sum_digits(0) != 0 || sum_digits(7) != 7)
        return 0;
    if (sum_digits(42) != 6 || sum_digits(1000) != 1)
        return 0;
    if (sum_digits(-1203) != 6 || sum_digits(99999) != 45)
        return 0;
    return SANDBOX_SUCCESS;
}
