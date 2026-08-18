extern int char_to_digit(char c);

int main(void)
{
    if (char_to_digit('0') != 0 || char_to_digit('5') != 5 || char_to_digit('9') != 9)
        return 0;
    if (char_to_digit('/') != -1 || char_to_digit(':') != -1 || char_to_digit('a') != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
