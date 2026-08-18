extern int sign(int n);

int main(void)
{
    if (sign(0) != 0 || sign(1) != 1 || sign(999) != 1)
        return 0;
    if (sign(-1) != -1 || sign(-999) != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
