extern int string_compare(char *a, char *b);

int main(void)
{
    if (string_compare((void *)0, (void *)0) != 0)
        return 0;
    if (string_compare((void *)0, "") != -1 || string_compare("", (void *)0) != 1)
        return 0;
    if (string_compare("", "") != 0 || string_compare("answer", "answer") != 0)
        return 0;
    if (string_compare("answer", "answers") != -1 || string_compare("answers", "answer") != 1)
        return 0;
    if (string_compare("abc", "abd") != -1 || string_compare("abd", "abc") != 1)
        return 0;
    return SANDBOX_SUCCESS;
}
