extern void increment_array(int *array, int size);

int main(void)
{
    int values[] = {-2, 0, 7, 99};

    increment_array((void *)0, 4);
    increment_array(values, 0);
    increment_array(values, -1);
    if (values[0] != -2 || values[3] != 99)
        return 0;
    increment_array(values, 3);
    if (values[0] != -1 || values[1] != 1 || values[2] != 8 || values[3] != 99)
        return 0;
    increment_array(values, 4);
    if (values[0] != 0 || values[1] != 2 || values[2] != 9 || values[3] != 100)
        return 0;
    return SANDBOX_SUCCESS;
}
