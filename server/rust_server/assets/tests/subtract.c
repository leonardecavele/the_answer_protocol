extern int subtract(int a, int b);

int main(void)
{
    if (subtract(0, 0) != 0 || subtract(8, 3) != 5)
        return 0;
    if (subtract(3, 8) != -5 || subtract(-3, -8) != 5)
        return 0;
    if (subtract(-3, 8) != -11 || subtract(3, -8) != 11)
        return 0;
    return SANDBOX_SUCCESS;
}
