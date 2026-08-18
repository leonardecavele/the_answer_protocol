extern int is_space(char c);

int main(void)
{
    if (is_space(' ') != 1 || is_space('\t') != 1 || is_space('\n') != 1)
        return 0;
    if (is_space('\r') != 1 || is_space('\v') != 1 || is_space('\f') != 1)
        return 0;
    if (is_space('a') != 0 || is_space('\0') != 0 || is_space('\b') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
