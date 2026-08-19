extern void fill_array(int *array, int size, int value);

int main(void)
{
    int values[] = {1, 2, 3, 4, 5};

    fill_array((void *)0, 5, 8);
    fill_array(values, 0, 8);
    fill_array(values, -2, 8);
    if (values[0] != 1 || values[4] != 5)
        return 0;
    fill_array(values, 3, -7);
    if (values[0] != -7 || values[1] != -7 || values[2] != -7)
        return 0;
    if (values[3] != 4 || values[4] != 5)
        return 0;
    fill_array(values, 5, 0);
    if (values[0] != 0 || values[1] != 0 || values[2] != 0 || values[3] != 0 || values[4] != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
