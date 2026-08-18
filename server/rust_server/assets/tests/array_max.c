extern int array_max(int *array, int size);

int main(void)
{
    int values[] = {-8, 4, 12, 12, -3};

    if (array_max((void *)0, 5) != 0 || array_max(values, 0) != 0)
        return 0;
    if (array_max(values, -1) != 0 || array_max(values, 1) != -8)
        return 0;
    if (array_max(values, 2) != 4 || array_max(values, 5) != 12)
        return 0;
    return SANDBOX_SUCCESS;
}
