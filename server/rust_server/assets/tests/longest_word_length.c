extern int longest_word_length(char *str);

int main(void)
{
    if (longest_word_length((void *)0) != 0 || longest_word_length("") != 0)
        return 0;
    if (longest_word_length("     ") != 0 || longest_word_length("a") != 1)
        return 0;
    if (longest_word_length("the answer protocol") != 8)
        return 0;
    if (longest_word_length("  small   enormous  word ") != 8)
        return 0;
    return SANDBOX_SUCCESS;
}
