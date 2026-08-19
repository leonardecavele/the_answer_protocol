extern int string_is_numeric(char *str);

int main(void)
{
    if (string_is_numeric((void *)0) != 0 || string_is_numeric("") != 0)
        return 0;
    if (string_is_numeric("0") != 1 || string_is_numeric("0123456789") != 1)
        return 0;
    if (string_is_numeric("42a") != 0 || string_is_numeric("-42") != 0)
        return 0;
    if (string_is_numeric("42 0") != 0 || string_is_numeric(" 42") != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
