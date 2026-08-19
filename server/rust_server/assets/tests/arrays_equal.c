extern int arrays_equal(int *a, int *b, int size);

int main(void)
{
    int a[] = {1, -2, 3, 4};
    int b[] = {1, -2, 9, 4};
    int c[] = {1, -2, 3, 4};

    if (arrays_equal((void *)0, (void *)0, 0) != 1 || arrays_equal(a, b, -2) != 1)
        return 0;
    if (arrays_equal((void *)0, b, 2) != 0 || arrays_equal(a, (void *)0, 2) != 0)
        return 0;
    if (arrays_equal(a, b, 2) != 1 || arrays_equal(a, b, 3) != 0)
        return 0;
    if (arrays_equal(a, c, 4) != 1)
        return 0;
    return SANDBOX_SUCCESS;
}
