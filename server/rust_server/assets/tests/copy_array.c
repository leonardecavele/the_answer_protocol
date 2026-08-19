extern int copy_array(int *dest, int *src, int size);

static int equal_values(int *a, int *b, int size)
{
    int i = 0;

    while (i < size)
    {
        if (a[i] != b[i])
            return 0;
        i++;
    }
    return 1;
}

int main(void)
{
    int src[] = {5, -2, 7, 0};
    int dest[] = {9, 9, 9, 9};
    int expected[] = {5, -2, 7, 9};

    if (copy_array((void *)0, src, 4) != 0 || copy_array(dest, (void *)0, 4) != 0)
        return 0;
    if (copy_array(dest, src, 0) != 0 || copy_array(dest, src, -2) != 0)
        return 0;
    if (copy_array(dest, src, 3) != 3 || !equal_values(dest, expected, 4))
        return 0;
    if (copy_array(dest, src, 4) != 4 || !equal_values(dest, src, 4))
        return 0;
    return SANDBOX_SUCCESS;
}
