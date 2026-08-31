extern int compare_ints(int a, int b);

int main(void)
{
    if (compare_ints(1, 2) != -1 || compare_ints(2, 1) != 1)
        return 0;
    if (compare_ints(0, 0) != 0 || compare_ints(-8, -8) != 0)
        return 0;
    if (compare_ints(-9, 4) != -1 || compare_ints(4, -9) != 1)
        return 0;
    return SANDBOX_SUCCESS;
}
