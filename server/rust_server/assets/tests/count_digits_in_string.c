extern int count_digits_in_string(char *str);

int main(void)
{
    if (count_digits_in_string((void *)0) != 0 || count_digits_in_string("") != 0)
        return 0;
    if (count_digits_in_string("answer") != 0 || count_digits_in_string("42") != 2)
        return 0;
    if (count_digits_in_string("room 101, floor -2") != 4)
        return 0;
    if (count_digits_in_string("0a1b2c3d4e5f6g7h8i9") != 10)
        return 0;
    return SANDBOX_SUCCESS;
}
