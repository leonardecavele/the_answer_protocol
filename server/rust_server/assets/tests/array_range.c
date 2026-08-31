extern int array_range(int *array, int size);

int main(void)
{
    int values[] = {8, -4, 12, 3, -10};
    int equal[] = {7, 7, 7};

    if (array_range((void *)0, 5) != 0 || array_range(values, 0) != 0)
        return 0;
    if (array_range(values, -2) != 0 || array_range(values, 1) != 0)
        return 0;
    if (array_range(values, 2) != 12 || array_range(values, 5) != 22)
        return 0;
    if (array_range(equal, 3) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
