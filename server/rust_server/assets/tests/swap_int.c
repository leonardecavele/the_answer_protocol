extern void swap_int(int *a, int *b);

int main(void)
{
    int a = 3;
    int b = -8;
    int unchanged = 42;

    swap_int(&a, &b);
    if (a != -8 || b != 3)
        return 0;
    swap_int(&a, &a);
    if (a != -8)
        return 0;
    swap_int((void *)0, &unchanged);
    swap_int(&unchanged, (void *)0);
    if (unchanged != 42)
        return 0;
    return SANDBOX_SUCCESS;
}
