extern char last_char(char *str);

int main(void)
{
    if (last_char((void *)0) != '\0' || last_char("") != '\0')
        return 0;
    if (last_char("x") != 'x' || last_char("answer") != 'r')
        return 0;
    if (last_char("with space ") != ' ' || last_char("42\n") != '\n')
        return 0;
    return SANDBOX_SUCCESS;
}
