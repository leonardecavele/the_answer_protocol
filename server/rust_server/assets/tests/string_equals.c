extern int string_equals(char *a, char *b);

int main(void)
{
    if (string_equals((void *)0, (void *)0) != 1)
        return 0;
    if (string_equals((void *)0, "") != 0 || string_equals("", (void *)0) != 0)
        return 0;
    if (string_equals("", "") != 1 || string_equals("answer", "answer") != 1)
        return 0;
    if (string_equals("answer", "Answer") != 0 || string_equals("answer", "answers") != 0)
        return 0;
    if (string_equals("answers", "answer") != 0 || string_equals("abc", "abd") != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
