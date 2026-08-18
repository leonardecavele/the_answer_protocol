extern char to_lowercase(char c);

int main(void)
{
    if (to_lowercase('A') != 'a' || to_lowercase('M') != 'm' || to_lowercase('Z') != 'z')
        return 0;
    if (to_lowercase('a') != 'a' || to_lowercase('z') != 'z')
        return 0;
    if (to_lowercase('@') != '@' || to_lowercase('[') != '[' || to_lowercase('0') != '0')
        return 0;
    return SANDBOX_SUCCESS;
}
