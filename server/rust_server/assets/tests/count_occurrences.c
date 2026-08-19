extern int count_occurrences(int *array, int size, int value);

int main(void)
{
    int values[] = {4, -2, 4, 0, 4, -2};

    if (count_occurrences((void *)0, 6, 4) != 0)
        return 0;
    if (count_occurrences(values, 0, 4) != 0 || count_occurrences(values, -2, 4) != 0)
        return 0;
    if (count_occurrences(values, 2, 4) != 1 || count_occurrences(values, 6, 4) != 3)
        return 0;
    if (count_occurrences(values, 6, -2) != 2 || count_occurrences(values, 6, 99) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
