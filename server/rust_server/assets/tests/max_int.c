extern int max_int(int a, int b);

int main(void)
{
    if (max_int(1, 2) != 2 || max_int(2, 1) != 2)
        return 0;
    if (max_int(-7, -3) != -3 || max_int(-3, -7) != -3)
        return 0;
    if (max_int(4, 4) != 4 || max_int(-1, 0) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
