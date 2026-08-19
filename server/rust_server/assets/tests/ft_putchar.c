extern void ft_putchar(char c);

static char captured[128];
static unsigned long captured_size;
static int invalid_write;

long write(int fd, const void *buffer, unsigned long count)
{
    const char *bytes = buffer;
    unsigned long i = 0;

    if (fd != 1 || buffer == (void *)0 || captured_size + count > sizeof(captured))
    {
        invalid_write = 1;
        return -1;
    }
    while (i < count)
    {
        captured[captured_size + i] = bytes[i];
        i++;
    }
    captured_size += count;
    return (long)count;
}

static int produced(char c)
{
    return !invalid_write && captured_size == 1 && captured[0] == c;
}

static void reset_capture(void)
{
    captured_size = 0;
    invalid_write = 0;
}

int main(void)
{
    ft_putchar('A');
    if (!produced('A'))
        return 0;
    reset_capture();
    ft_putchar('\n');
    if (!produced('\n'))
        return 0;
    reset_capture();
    ft_putchar('\0');
    if (!produced('\0'))
        return 0;
    return SANDBOX_SUCCESS;
}
