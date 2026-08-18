extern int is_alpha(char c);

int main(void)
{
    if (is_alpha('a') != 1 || is_alpha('z') != 1)
        return 0;
    if (is_alpha('A') != 1 || is_alpha('Z') != 1 || is_alpha('m') != 1)
        return 0;
    if (is_alpha('0') != 0 || is_alpha('@') != 0 || is_alpha('[') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
