extern int count_substring(char *str, char *needle);

int main(void)
{
    if (count_substring((void *)0, "a") != 0 || count_substring("a", (void *)0) != 0)
        return 0;
    if (count_substring("answer", "") != 0 || count_substring("", "a") != 0)
        return 0;
    if (count_substring("banana", "ana") != 2 || count_substring("aaaa", "aa") != 3)
        return 0;
    if (count_substring("the answer", "answer") != 1)
        return 0;
    if (count_substring("short", "longer") != 0 || count_substring("abc", "z") != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
