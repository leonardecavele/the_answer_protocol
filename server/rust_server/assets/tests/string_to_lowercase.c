extern void string_to_lowercase(char *str);

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
    char upper[] = "ANSWER";
    char mixed[] = "42 Hello, Z!";

    string_to_lowercase((void *)0);
    string_to_lowercase(empty);
    string_to_lowercase(upper);
    string_to_lowercase(mixed);
    if (!equals(empty, "") || !equals(upper, "answer"))
        return 0;
    if (!equals(mixed, "42 hello, z!"))
        return 0;
    return SANDBOX_SUCCESS;
}
