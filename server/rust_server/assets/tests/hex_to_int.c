extern int hex_to_int(char c);

int main(void)
{
    if (hex_to_int('0') != 0 || hex_to_int('9') != 9)
        return 0;
    if (hex_to_int('a') != 10 || hex_to_int('f') != 15)
        return 0;
    if (hex_to_int('A') != 10 || hex_to_int('F') != 15)
        return 0;
    if (hex_to_int('/') != -1 || hex_to_int(':') != -1)
        return 0;
    if (hex_to_int('g') != -1 || hex_to_int('G') != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
