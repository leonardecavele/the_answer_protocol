extern int alphabet_position(char c);

int main(void)
{
    if (alphabet_position('a') != 1 || alphabet_position('A') != 1)
        return 0;
    if (alphabet_position('m') != 13 || alphabet_position('Z') != 26)
        return 0;
    if (alphabet_position('@') != -1 || alphabet_position('[') != -1)
        return 0;
    if (alphabet_position('0') != -1 || alphabet_position(' ') != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
