#include "libft.h"

char	*ft_strchr(const char *s, int c)
{
	while (*s)
	{
		if ((uint8_t)(*s) == (uint8_t)c)
			return ((char *)s);
		s++;
	}
	if (!(uint8_t)c)
		return ((char *)s);
	return (0);
}
