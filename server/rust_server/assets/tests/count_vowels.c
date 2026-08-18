extern int count_vowels(char *str);

int main(void)
{
    if (count_vowels((void *)0) != 0 || count_vowels("") != 0)
        return 0;
    if (count_vowels("bcdfg") != 0 || count_vowels("answer") != 2)
        return 0;
    if (count_vowels("AEIOUaeiou") != 10 || count_vowels("42 Umbrella!") != 3)
        return 0;
    return SANDBOX_SUCCESS;
}
