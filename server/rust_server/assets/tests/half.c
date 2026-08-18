extern int half(int n);

int main(void)
{
    if (half(0) != 0 || half(1) != 0 || half(-1) != 0)
        return 0;
    if (half(8) != 4 || half(5) != 2)
        return 0;
    if (half(-8) != -4 || half(-5) != -2)
        return 0;
    return SANDBOX_SUCCESS;
}
