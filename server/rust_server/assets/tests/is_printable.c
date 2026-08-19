extern int is_printable(char c);

int main(void)
{
    if (is_printable(' ') != 1 || is_printable('A') != 1 || is_printable('~') != 1)
        return 0;
    if (is_printable('\0') != 0 || is_printable('\t') != 0 || is_printable('\n') != 0)
        return 0;
    if (is_printable(31) != 0 || is_printable(127) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
