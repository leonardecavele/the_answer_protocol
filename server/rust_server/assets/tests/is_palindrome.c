extern int is_palindrome(char *str);

int main(void)
{
    if (is_palindrome((void *)0) != 0 || is_palindrome("") != 1)
        return 0;
    if (is_palindrome("a") != 1 || is_palindrome("abba") != 1)
        return 0;
    if (is_palindrome("racecar") != 1 || is_palindrome("answer") != 0)
        return 0;
    if (is_palindrome("Racecar") != 0 || is_palindrome("abca") != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
