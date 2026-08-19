extern void ft_putendl(char *str);

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

static int matches(char *expected)
{
    unsigned long i = 0;

    if (invalid_write)
        return 0;
    while (expected[i])
    {
        if (i >= captured_size || captured[i] != expected[i])
            return 0;
        i++;
    }
    return i == captured_size;
}

static void reset_capture(void)
{
    captured_size = 0;
    invalid_write = 0;
}

int main(void)
{
    ft_putendl((void *)0);
    if (!matches(""))
        return 0;
    reset_capture();
    ft_putendl("");
    if (!matches("\n"))
        return 0;
    reset_capture();
    ft_putendl("answer");
    if (!matches("answer\n"))
        return 0;
    reset_capture();
    ft_putendl("already\n");
    if (!matches("already\n\n"))
        return 0;
    return SANDBOX_SUCCESS;
}
