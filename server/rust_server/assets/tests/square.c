extern int square(int n);

int main(void)
{
    if (square(0) != 0 || square(1) != 1 || square(-1) != 1)
        return 0;
    if (square(12) != 144 || square(-12) != 144)
        return 0;
    if (square(123) != 15129)
        return 0;
    return SANDBOX_SUCCESS;
}
