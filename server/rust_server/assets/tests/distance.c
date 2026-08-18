extern int distance(int a, int b);

int main(void)
{
    if (distance(0, 0) != 0 || distance(3, 10) != 7)
        return 0;
    if (distance(10, 3) != 7 || distance(-8, -3) != 5)
        return 0;
    if (distance(-8, 3) != 11 || distance(8, -3) != 11)
        return 0;
    return SANDBOX_SUCCESS;
}
