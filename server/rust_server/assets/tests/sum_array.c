extern int sum_array(int *array, int size);

int main(void)
{
    int values[] = {5, -2, 7, -10, 4};

    if (sum_array((void *)0, 5) != 0 || sum_array(values, 0) != 0)
        return 0;
    if (sum_array(values, -1) != 0 || sum_array(values, 1) != 5)
        return 0;
    if (sum_array(values, 3) != 10 || sum_array(values, 5) != 4)
        return 0;
    return SANDBOX_SUCCESS;
}
