extern int ft_strlen(char *str);

int main(void)
{
    if (ft_strlen((void *)0) != 0 || ft_strlen("") != 0)
        return 0;
    if (ft_strlen("a") != 1 || ft_strlen("answer") != 6)
        return 0;
    if (ft_strlen("with spaces") != 11 || ft_strlen("42\n") != 3)
        return 0;
    return SANDBOX_SUCCESS;
}
