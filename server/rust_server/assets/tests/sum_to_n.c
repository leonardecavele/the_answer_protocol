extern int sum_to_n(int n);

int main(void)
{
    if (sum_to_n(-10) != 0 || sum_to_n(0) != 0)
        return 0;
    if (sum_to_n(1) != 1 || sum_to_n(2) != 3)
        return 0;
    if (sum_to_n(10) != 55 || sum_to_n(100) != 5050)
        return 0;
    if (sum_to_n(1000) != 500500)
        return 0;
    return SANDBOX_SUCCESS;
}
