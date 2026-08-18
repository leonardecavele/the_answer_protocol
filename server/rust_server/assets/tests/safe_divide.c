extern int safe_divide(int a, int b);

int main(void)
{
    if (safe_divide(7, 0) != 0 || safe_divide(0, 3) != 0)
        return 0;
    if (safe_divide(8, 4) != 2 || safe_divide(7, 3) != 2)
        return 0;
    if (safe_divide(-7, 3) != -2 || safe_divide(7, -3) != -2)
        return 0;
    if (safe_divide(-7, -3) != 2)
        return 0;
    return SANDBOX_SUCCESS;
}
