extern int ends_with(char *str, char c);

int main(void)
{
    if (ends_with((void *)0, 'a') != 0 || ends_with("", 'a') != 0)
        return 0;
    if (ends_with("answer", 'r') != 1 || ends_with("answer", 'a') != 0)
        return 0;
    if (ends_with("x", 'x') != 1 || ends_with("x", 'y') != 0)
        return 0;
    if (ends_with("answer", '\0') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
