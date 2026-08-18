extern int is_alphanumeric(char c);

int main(void)
{
    if (is_alphanumeric('a') != 1 || is_alphanumeric('Z') != 1)
        return 0;
    if (is_alphanumeric('0') != 1 || is_alphanumeric('9') != 1)
        return 0;
    if (is_alphanumeric('/') != 0 || is_alphanumeric(':') != 0)
        return 0;
    if (is_alphanumeric('@') != 0 || is_alphanumeric('[') != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
