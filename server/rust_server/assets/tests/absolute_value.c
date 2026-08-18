extern int absolute_value(int n);

int main(void)
{
    if (absolute_value(0) != 0 || absolute_value(1) != 1)
        return 0;
    if (absolute_value(-1) != 1 || absolute_value(-456) != 456)
        return 0;
    return SANDBOX_SUCCESS;
}
