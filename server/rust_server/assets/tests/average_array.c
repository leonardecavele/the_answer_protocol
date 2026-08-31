extern int average_array(int *array, int size);

int main(void)
{
    int positive[] = {2, 5, 8, 9};
    int mixed[] = {-8, 3, 2};

    if (average_array((void *)0, 4) != 0 || average_array(positive, 0) != 0)
        return 0;
    if (average_array(positive, -1) != 0 || average_array(positive, 1) != 2)
        return 0;
    if (average_array(positive, 3) != 5 || average_array(positive, 4) != 6)
        return 0;
    if (average_array(mixed, 3) != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
