int	ft_isascii(int c)
{
	int	i;
	int	is_ascii;

	i = 0;
	is_ascii = 0;
	while (++i <= 127)
		if (c == i)
			is_ascii = 1;
	return (is_ascii);
}