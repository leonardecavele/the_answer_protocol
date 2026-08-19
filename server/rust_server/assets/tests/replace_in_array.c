extern int replace_in_array(int *array, int size, int from, int to);

int main(void)
{
    int values[] = {4, -2, 4, 0, 4};

    if (replace_in_array((void *)0, 5, 4, 9) != 0)
        return 0;
    if (replace_in_array(values, 0, 4, 9) != 0 || replace_in_array(values, -1, 4, 9) != 0)
        return 0;
    if (replace_in_array(values, 3, 4, 9) != 2)
        return 0;
    if (values[0] != 9 || values[1] != -2 || values[2] != 9 || values[3] != 0 || values[4] != 4)
        return 0;
    if (replace_in_array(values, 5, 4, 9) != 1 || values[4] != 9)
        return 0;
    if (replace_in_array(values, 5, 42, 1) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
