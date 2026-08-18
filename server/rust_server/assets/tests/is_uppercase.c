extern int is_uppercase(char c);

int main(void)
{
    if (is_uppercase('A') != 1 || is_uppercase('M') != 1 || is_uppercase('Z') != 1)
        return 0;
    if (is_uppercase('@') != 0 || is_uppercase('[') != 0)
        return 0;
    if (is_uppercase('a') != 0 || is_uppercase('0') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
