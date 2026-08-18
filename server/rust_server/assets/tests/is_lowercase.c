extern int is_lowercase(char c);

int main(void)
{
    if (is_lowercase('a') != 1 || is_lowercase('m') != 1 || is_lowercase('z') != 1)
        return 0;
    if (is_lowercase('`') != 0 || is_lowercase('{') != 0)
        return 0;
    if (is_lowercase('A') != 0 || is_lowercase('0') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
