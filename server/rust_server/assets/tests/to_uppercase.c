extern char to_uppercase(char c);

int main(void)
{
    if (to_uppercase('a') != 'A' || to_uppercase('m') != 'M' || to_uppercase('z') != 'Z')
        return 0;
    if (to_uppercase('A') != 'A' || to_uppercase('Z') != 'Z')
        return 0;
    if (to_uppercase('`') != '`' || to_uppercase('{') != '{' || to_uppercase('0') != '0')
        return 0;
    return SANDBOX_SUCCESS;
}
