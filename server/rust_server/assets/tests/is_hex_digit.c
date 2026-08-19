extern int is_hex_digit(char c);

int main(void)
{
    if (is_hex_digit('0') != 1 || is_hex_digit('9') != 1)
        return 0;
    if (is_hex_digit('a') != 1 || is_hex_digit('f') != 1)
        return 0;
    if (is_hex_digit('A') != 1 || is_hex_digit('F') != 1)
        return 0;
    if (is_hex_digit('/') != 0 || is_hex_digit(':') != 0)
        return 0;
    if (is_hex_digit('g') != 0 || is_hex_digit('G') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
