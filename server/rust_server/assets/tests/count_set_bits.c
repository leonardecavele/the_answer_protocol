extern int count_set_bits(unsigned int n);

int main(void)
{
    if (count_set_bits(0) != 0 || count_set_bits(1) != 1)
        return 0;
    if (count_set_bits(2) != 1 || count_set_bits(3) != 2)
        return 0;
    if (count_set_bits(0xf0u) != 4 || count_set_bits(0x80000000u) != 1)
        return 0;
    if (count_set_bits(0xffffffffu) != 32)
        return 0;
    return SANDBOX_SUCCESS;
}
