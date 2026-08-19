extern int count_digits(int n);

int main(void)
{
    if (count_digits(0) != 1 || count_digits(7) != 1 || count_digits(-7) != 1)
        return 0;
    if (count_digits(10) != 2 || count_digits(-999) != 3)
        return 0;
    if (count_digits(1000000) != 7 || count_digits(-123456789) != 9)
        return 0;
    return SANDBOX_SUCCESS;
}
