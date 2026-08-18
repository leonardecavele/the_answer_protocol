extern int is_sorted_ascending(int *array, int size);

int main(void)
{
    int sorted[] = {-5, -1, -1, 0, 8, 12};
    int unsorted[] = {1, 3, 2, 4};

    if (is_sorted_ascending((void *)0, 0) != 1)
        return 0;
    if (is_sorted_ascending((void *)0, 1) != 1 || is_sorted_ascending((void *)0, 2) != 0)
        return 0;
    if (is_sorted_ascending(sorted, 6) != 1 || is_sorted_ascending(unsorted, 4) != 0)
        return 0;
    if (is_sorted_ascending(unsorted, 2) != 1 || is_sorted_ascending(sorted, -3) != 1)
        return 0;
    return SANDBOX_SUCCESS;
}
