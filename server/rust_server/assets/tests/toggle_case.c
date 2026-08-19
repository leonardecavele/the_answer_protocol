extern char toggle_case(char c);

int main(void)
{
    if (toggle_case('a') != 'A' || toggle_case('m') != 'M' || toggle_case('z') != 'Z')
        return 0;
    if (toggle_case('A') != 'a' || toggle_case('M') != 'm' || toggle_case('Z') != 'z')
        return 0;
    if (toggle_case('0') != '0' || toggle_case('@') != '@' || toggle_case(' ') != ' ')
        return 0;
    return SANDBOX_SUCCESS;
}
