extern int array_contains(int *array, int size, int value);

int main(void)
{
    int values[] = {4, -2, 7, 7, 0};

    if (array_contains((void *)0, 5, 4) != 0)
        return 0;
    if (array_contains(values, 0, 4) != 0 || array_contains(values, -2, 4) != 0)
        return 0;
    if (array_contains(values, 5, 4) != 1 || array_contains(values, 5, 7) != 1)
        return 0;
    if (array_contains(values, 4, 0) != 0 || array_contains(values, 5, 9) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
