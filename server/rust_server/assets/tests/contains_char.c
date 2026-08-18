extern int contains_char(char *str, char c);

int main(void)
{
    if (contains_char((void *)0, 'a') != 0 || contains_char("", 'a') != 0)
        return 0;
    if (contains_char("answer", 'a') != 1 || contains_char("answer", 's') != 1)
        return 0;
    if (contains_char("answer", 'r') != 1 || contains_char("answer", 'z') != 0)
        return 0;
    if (contains_char("", '\0') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
