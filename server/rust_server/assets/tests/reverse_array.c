extern void reverse_array(int *array, int size);

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
    int even[] = {1, 2, 3, 4};
    int odd[] = {-2, 0, 9, 5, 8};
    int expected_even[] = {4, 3, 2, 1};
    int expected_odd[] = {8, 5, 9, 0, -2};

    reverse_array((void *)0, 4);
    reverse_array(one, 1);
    reverse_array(even, 4);
    reverse_array(odd, 5);
    if (one[0] != 7 || !equal_values(even, expected_even, 4))
        return 0;
    if (!equal_values(odd, expected_odd, 5))
        return 0;
    return SANDBOX_SUCCESS;
}
