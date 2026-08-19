extern int string_concat(char *dest, char *src);

static int equals(char *a, char *b)
{
    int i = 0;

    while (a[i] && a[i] == b[i])
        i++;
    return a[i] == b[i];
}

int main(void)
{
    char empty[8] = "";
    char first[32] = "the";
    char unchanged[8] = "safe";

    if (string_concat((void *)0, "x") != 0)
        return 0;
    if (string_concat(unchanged, (void *)0) != 0 || !equals(unchanged, "safe"))
        return 0;
    if (string_concat(empty, "answer") != 6 || !equals(empty, "answer"))
        return 0;
    if (string_concat(first, " answer") != 10 || !equals(first, "the answer"))
        return 0;
    if (string_concat(first, "") != 10 || !equals(first, "the answer"))
        return 0;
    return SANDBOX_SUCCESS;
}
