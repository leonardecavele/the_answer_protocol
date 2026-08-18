extern int count_positive(int *array, int size);

int main(void)
{
    int values[] = {4, 0, -7, 1, 9, -2};

    if (count_positive((void *)0, 6) != 0 || count_positive(values, 0) != 0)
        return 0;
    if (count_positive(values, -2) != 0 || count_positive(values, 1) != 1)
        return 0;
    if (count_positive(values, 3) != 1 || count_positive(values, 6) != 3)
        return 0;
    return SANDBOX_SUCCESS;
}
