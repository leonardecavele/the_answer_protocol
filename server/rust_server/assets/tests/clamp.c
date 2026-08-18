extern int clamp(int value, int min, int max);

int main(void)
{
    if (clamp(-4, -2, 6) != -2 || clamp(-2, -2, 6) != -2)
        return 0;
    if (clamp(3, -2, 6) != 3 || clamp(6, -2, 6) != 6)
        return 0;
    if (clamp(10, -2, 6) != 6 || clamp(8, 8, 8) != 8)
        return 0;
    return SANDBOX_SUCCESS;
}
