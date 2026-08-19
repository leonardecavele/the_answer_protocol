extern void string_to_uppercase(char *str);

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
    char lower[] = "answer";
    char mixed[] = "42 Hello, z!";

    string_to_uppercase((void *)0);
    string_to_uppercase(empty);
    string_to_uppercase(lower);
    string_to_uppercase(mixed);
    if (!equals(empty, "") || !equals(lower, "ANSWER"))
        return 0;
    if (!equals(mixed, "42 HELLO, Z!"))
        return 0;
    return SANDBOX_SUCCESS;
}
