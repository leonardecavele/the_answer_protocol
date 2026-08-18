extern int count_even(int *array, int size);

int main(void)
{
    int values[] = {-4, -3, 0, 5, 8, 11};

    if (count_even((void *)0, 6) != 0 || count_even(values, 0) != 0)
        return 0;
    if (count_even(values, -2) != 0 || count_even(values, 1) != 1)
        return 0;
    if (count_even(values, 4) != 2 || count_even(values, 6) != 3)
        return 0;
    return SANDBOX_SUCCESS;
}
