extern int is_multiple(int value, int divisor);

int main(void)
{
    if (is_multiple(5, 0) != 0 || is_multiple(0, 0) != 0)
        return 0;
    if (is_multiple(0, 7) != 1 || is_multiple(12, 3) != 1)
        return 0;
    if (is_multiple(-12, 3) != 1 || is_multiple(12, -3) != 1)
        return 0;
    if (is_multiple(13, 3) != 0 || is_multiple(-13, 3) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
