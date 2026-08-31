extern void rotate_left(int *array, int size);

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
    int one[] = {7};
    int values[] = {1, 2, 3, 4, 99};
    int expected_four[] = {2, 3, 4, 1, 99};
    int expected_five[] = {3, 4, 1, 99, 2};

    rotate_left((void *)0, 4);
    rotate_left(one, 1);
    rotate_left(values, 4);
    if (one[0] != 7 || !equal_values(values, expected_four, 5))
        return 0;
    rotate_left(values, 5);
    if (!equal_values(values, expected_five, 5))
        return 0;
    return SANDBOX_SUCCESS;
}
