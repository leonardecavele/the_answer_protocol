extern int replace_char(char *str, char from, char to);

static int equals(char *a, char *b)
{
    int i = 0;

    while (a[i] && a[i] == b[i])
        i++;
    return a[i] == b[i];
}

int main(void)
{
    char empty[] = "";
    char word[] = "banana";
    char none[] = "answer";

    if (replace_char((void *)0, 'a', 'x') != 0)
        return 0;
    if (replace_char(empty, 'a', 'x') != 0 || replace_char(none, 'z', 'x') != 0)
        return 0;
    if (replace_char(word, 'a', 'o') != 3 || !equals(word, "bonono"))
        return 0;
    if (replace_char(word, '\0', 'x') != 0 || !equals(word, "bonono"))
        return 0;
    return SANDBOX_SUCCESS;
}
