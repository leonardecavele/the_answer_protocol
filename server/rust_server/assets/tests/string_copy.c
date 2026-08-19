extern int string_copy(char *dest, char *src);

static int equals(char *a, char *b)
{
    int i = 0;

    while (a[i] && a[i] == b[i])
        i++;
    return a[i] == b[i];
}

int main(void)
{
    char empty[4] = {'x', 'x', 'x', '\0'};
    char word[16] = {0};
    char untouched[4] = "ok";

    if (string_copy((void *)0, "abc") != 0 || string_copy(untouched, (void *)0) != 0)
        return 0;
    if (!equals(untouched, "ok"))
        return 0;
    if (string_copy(empty, "") != 0 || !equals(empty, ""))
        return 0;
    if (string_copy(word, "the answer") != 10 || !equals(word, "the answer"))
        return 0;
    return SANDBOX_SUCCESS;
}
