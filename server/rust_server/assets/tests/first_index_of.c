extern int first_index_of(int *array, int size, int value);

int main(void)
{
    int values[] = {5, -2, 7, -2, 9};

    if (first_index_of((void *)0, 5, 5) != -1 || first_index_of(values, 0, 5) != -1)
        return 0;
    if (first_index_of(values, -1, 5) != -1 || first_index_of(values, 5, -2) != 1)
        return 0;
    if (first_index_of(values, 3, 9) != -1 || first_index_of(values, 5, 9) != 4)
        return 0;
    if (first_index_of(values, 5, 42) != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
