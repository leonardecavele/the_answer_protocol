extern int fibonacci(int n);

int main(void)
{
    if (fibonacci(-5) != 0 || fibonacci(0) != 0 || fibonacci(1) != 1)
        return 0;
    if (fibonacci(2) != 1 || fibonacci(5) != 5 || fibonacci(10) != 55)
        return 0;
    if (fibonacci(20) != 6765 || fibonacci(46) != 1836311903)
        return 0;
    return SANDBOX_SUCCESS;
}
