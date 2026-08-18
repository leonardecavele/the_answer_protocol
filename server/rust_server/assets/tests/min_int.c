extern int min_int(int a, int b);

int main(void)
{
    if (min_int(1, 2) != 1 || min_int(2, 1) != 1)
        return 0;
    if (min_int(-7, -3) != -7 || min_int(-3, -7) != -7)
        return 0;
    if (min_int(4, 4) != 4 || min_int(-1, 0) != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
