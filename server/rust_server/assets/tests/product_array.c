extern int product_array(int *array, int size);

int main(void)
{
    int values[] = {-2, 3, 4, 0, 5};

    if (product_array((void *)0, 5) != 0 || product_array(values, 0) != 0)
        return 0;
    if (product_array(values, -2) != 0 || product_array(values, 1) != -2)
        return 0;
    if (product_array(values, 2) != -6 || product_array(values, 3) != -24)
        return 0;
    if (product_array(values, 5) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
