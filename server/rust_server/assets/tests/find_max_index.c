extern int find_max_index(int *array, int size);

int main(void)
{
    int values[] = {-8, 12, 4, 12, 3};

    if (find_max_index((void *)0, 5) != -1 || find_max_index(values, 0) != -1)
        return 0;
    if (find_max_index(values, -1) != -1 || find_max_index(values, 1) != 0)
        return 0;
    if (find_max_index(values, 3) != 1 || find_max_index(values, 5) != 1)
        return 0;
    return SANDBOX_SUCCESS;
}
