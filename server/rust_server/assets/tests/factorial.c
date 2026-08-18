extern int factorial(int n);

int main(void)
{
    if (factorial(-1) != 0 || factorial(-5) != 0)
        return 0;
    if (factorial(0) != 1 || factorial(1) != 1 || factorial(2) != 2)
        return 0;
    if (factorial(5) != 120 || factorial(8) != 40320)
        return 0;
    if (factorial(12) != 479001600)
        return 0;
    return SANDBOX_SUCCESS;
}
