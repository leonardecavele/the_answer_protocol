extern char digit_to_char(int n);

int main(void)
{
    if (digit_to_char(0) != '0' || digit_to_char(5) != '5' || digit_to_char(9) != '9')
        return 0;
    if (digit_to_char(-1) != '\0' || digit_to_char(10) != '\0')
        return 0;
    if (digit_to_char(100) != '\0')
        return 0;
    return SANDBOX_SUCCESS;
}
