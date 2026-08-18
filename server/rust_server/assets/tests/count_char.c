extern int count_char(char *str, char c);

int main(void)
{
    if (count_char((void *)0, 'a') != 0 || count_char("", 'a') != 0)
        return 0;
    if (count_char("banana", 'a') != 3 || count_char("banana", 'n') != 2)
        return 0;
    if (count_char("banana", 'b') != 1 || count_char("banana", 'z') != 0)
        return 0;
    if (count_char("banana", '\0') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
