extern int dot_product(int *a, int *b, int size);

int main(void)
{
    int a[] = {1, 2, -3, 4};
    int b[] = {5, -2, 3, 0};

    if (dot_product((void *)0, b, 4) != 0 || dot_product(a, (void *)0, 4) != 0)
        return 0;
    if (dot_product(a, b, 0) != 0 || dot_product(a, b, -1) != 0)
        return 0;
    if (dot_product(a, b, 1) != 5 || dot_product(a, b, 2) != 1)
        return 0;
    if (dot_product(a, b, 3) != -8 || dot_product(a, b, 4) != -8)
        return 0;
    return SANDBOX_SUCCESS;
}
