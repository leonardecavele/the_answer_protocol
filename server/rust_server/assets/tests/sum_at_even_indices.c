extern int sum_at_even_indices(int *array, int size);

int main(void)
{
    int values[] = {5, 100, -2, 200, 7, 300};

    if (sum_at_even_indices((void *)0, 6) != 0)
        return 0;
    if (sum_at_even_indices(values, 0) != 0 || sum_at_even_indices(values, -1) != 0)
        return 0;
    if (sum_at_even_indices(values, 1) != 5 || sum_at_even_indices(values, 2) != 5)
        return 0;
    if (sum_at_even_indices(values, 5) != 10 || sum_at_even_indices(values, 6) != 10)
        return 0;
    return SANDBOX_SUCCESS;
}
