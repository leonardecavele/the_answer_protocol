#include "libft.h"

char	*ft_strnstr(const char *haystack, const char *needle, size_t len)
{
	size_t	needle_len;
	size_t	i;

	if (!(*needle))
		return ((char *)haystack);
	if (!len)
		return (NULL);
	needle_len = ft_strlen(needle);
	i = -1;
	while (haystack[++i] && i + needle_len <= len)
		if (ft_strncmp(haystack + i, needle, needle_len) == 0)
			return ((char *)haystack + i);
	return (0);
}
