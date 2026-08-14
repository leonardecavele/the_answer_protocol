extern int array_min(int *array, int size);

int main(void)
{
    int values[] = {8, -4, -12, -12, 3};

    if (array_min((void *)0, 5) != 0 || array_min(values, 0) != 0)
        return 0;
    if (array_min(values, -1) != 0 || array_min(values, 1) != 8)
        return 0;
    if (array_min(values, 2) != -4 || array_min(values, 5) != -12)
        return 0;
    return SANDBOX_SUCCESS;
}
