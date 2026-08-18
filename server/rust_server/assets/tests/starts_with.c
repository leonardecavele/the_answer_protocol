extern int starts_with(char *str, char c);

int main(void)
{
    if (starts_with((void *)0, 'a') != 0 || starts_with("", 'a') != 0)
        return 0;
    if (starts_with("answer", 'a') != 1 || starts_with("answer", 'r') != 0)
        return 0;
    if (starts_with("x", 'x') != 1 || starts_with("x", 'y') != 0)
        return 0;
    if (starts_with("answer", '\0') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
