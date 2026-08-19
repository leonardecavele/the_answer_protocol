extern void reverse_string(char *str);

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
    char one[] = "x";
    char even[] = "abcd";
    char odd[] = "answer!";

    reverse_string((void *)0);
    reverse_string(empty);
    reverse_string(one);
    reverse_string(even);
    reverse_string(odd);
    if (!equals(empty, "") || !equals(one, "x"))
        return 0;
    if (!equals(even, "dcba") || !equals(odd, "!rewsna"))
        return 0;
    return SANDBOX_SUCCESS;
}
