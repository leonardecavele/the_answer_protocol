extern int count_words(char *str);

int main(void)
{
    if (count_words((void *)0) != 0 || count_words("") != 0)
        return 0;
    if (count_words("answer") != 1 || count_words("the answer") != 2)
        return 0;
    if (count_words("  the   answer  protocol ") != 3)
        return 0;
    if (count_words("     ") != 0 || count_words("a b c d") != 4)
        return 0;
    return SANDBOX_SUCCESS;
}
