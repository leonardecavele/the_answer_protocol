extern int add_one(int n);

int main(void)
{
    if (add_one(0) != 1 || add_one(-1) != 0)
        return 0;
    if (add_one(41) != 42 || add_one(-100) != -99)
        return 0;
    return SANDBOX_SUCCESS;
}
