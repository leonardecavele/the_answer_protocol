extern int is_prime(int n);

int main(void)
{
    if (is_prime(-7) != 0 || is_prime(0) != 0 || is_prime(1) != 0)
        return 0;
    if (is_prime(2) != 1 || is_prime(3) != 1 || is_prime(97) != 1)
        return 0;
    if (is_prime(4) != 0 || is_prime(49) != 0 || is_prime(100) != 0)
        return 0;
    return SANDBOX_SUCCESS;
}
