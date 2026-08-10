#include "libft.h"

int	ft_strncmp(const char *s1, const char *s2, size_t n)
{
	size_t	i;

	i = 0;
	if (!n)
		return (0);
	while (s1[i] && i + 1 < n && (uint8_t)s1[i] == (uint8_t)s2[i])
		i++;
	return ((uint8_t)s1[i] - (uint8_t)s2[i]);
}
