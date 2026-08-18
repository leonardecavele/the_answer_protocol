extern int is_between(int value, int min, int max);

int main(void)
{
    if (is_between(-3, -2, 5) != 0 || is_between(-2, -2, 5) != 1)
        return 0;
    if (is_between(0, -2, 5) != 1 || is_between(5, -2, 5) != 1)
        return 0;
    if (is_between(6, -2, 5) != 0 || is_between(8, 8, 8) != 1)
        return 0;
    return SANDBOX_SUCCESS;
}
