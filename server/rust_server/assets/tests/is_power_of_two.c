extern int is_power_of_two(unsigned int n);

int main(void)
{
    if (is_power_of_two(0) != 0 || is_power_of_two(1) != 1)
        return 0;
    if (is_power_of_two(2) != 1 || is_power_of_two(16) != 1 || is_power_of_two(1024) != 1)
        return 0;
    if (is_power_of_two(3) != 0 || is_power_of_two(12) != 0 || is_power_of_two(1023) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
