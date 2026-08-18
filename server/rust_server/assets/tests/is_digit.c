extern int is_digit(char c);

int main(void)
{
    if (is_digit('0') != 1 || is_digit('5') != 1 || is_digit('9') != 1)
        return 0;
    if (is_digit('/') != 0 || is_digit(':') != 0)
        return 0;
    if (is_digit('a') != 0 || is_digit(' ') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
