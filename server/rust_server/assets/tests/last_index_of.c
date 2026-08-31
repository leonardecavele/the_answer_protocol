extern int last_index_of(int *array, int size, int value);

int main(void)
{
    int values[] = {5, -2, 7, -2, 5};

    if (last_index_of((void *)0, 5, 5) != -1 || last_index_of(values, 0, 5) != -1)
        return 0;
    if (last_index_of(values, -1, 5) != -1 || last_index_of(values, 5, 42) != -1)
        return 0;
    if (last_index_of(values, 2, 5) != 0 || last_index_of(values, 5, -2) != 3)
        return 0;
    if (last_index_of(values, 5, 5) != 4)
        return 0;
    return SANDBOX_SUCCESS;
}
