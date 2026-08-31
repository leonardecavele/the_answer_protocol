extern int remove_spaces(char *str);

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
    char spaces[] = "    ";
    char text[] = " the  answer protocol ";
    char tabs[] = "a\tb\nc";

    if (remove_spaces((void *)0) != 0 || remove_spaces(empty) != 0)
        return 0;
    if (remove_spaces(spaces) != 0 || !equals(spaces, ""))
        return 0;
    if (remove_spaces(text) != 17 || !equals(text, "theanswerprotocol"))
        return 0;
    if (remove_spaces(tabs) != 5 || !equals(tabs, "a\tb\nc"))
        return 0;
    return SANDBOX_SUCCESS;
}
