extern int multiply(int a, int b);

int main(void)
{
    if (multiply(0, 99) != 0 || multiply(7, 6) != 42)
        return 0;
    if (multiply(-7, 6) != -42 || multiply(7, -6) != -42)
        return 0;
    if (multiply(-7, -6) != 42 || multiply(13, 11) != 143)
        return 0;
    return SANDBOX_SUCCESS;
}
