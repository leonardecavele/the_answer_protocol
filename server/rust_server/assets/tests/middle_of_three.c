extern int middle_of_three(int a, int b, int c);

int main(void)
{
    if (middle_of_three(1, 2, 3) != 2 || middle_of_three(1, 3, 2) != 2)
        return 0;
    if (middle_of_three(2, 1, 3) != 2 || middle_of_three(2, 3, 1) != 2)
        return 0;
    if (middle_of_three(3, 1, 2) != 2 || middle_of_three(3, 2, 1) != 2)
        return 0;
    if (middle_of_three(4, 4, 9) != 4 || middle_of_three(4, 9, 9) != 9)
        return 0;
    if (middle_of_three(-5, -5, -5) != -5)
        return 0;
    return SANDBOX_SUCCESS;
}
