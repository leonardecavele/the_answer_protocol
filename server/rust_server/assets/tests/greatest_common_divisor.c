extern int greatest_common_divisor(int a, int b);

int main(void)
{
    if (greatest_common_divisor(0, 0) != 0)
        return 0;
    if (greatest_common_divisor(0, 9) != 9 || greatest_common_divisor(12, 0) != 12)
        return 0;
    if (greatest_common_divisor(18, 24) != 6 || greatest_common_divisor(17, 13) != 1)
        return 0;
    if (greatest_common_divisor(-54, 24) != 6 || greatest_common_divisor(-42, -56) != 14)
        return 0;
    return SANDBOX_SUCCESS;
}
