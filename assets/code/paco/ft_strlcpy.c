#include "libft.h"

size_t	ft_strlcpy(char *dst, const char *src, size_t dstsize)
{
	size_t	i;

	i = 0;
	while (i + 1 < dstsize && src[i])
		*dst++ = src[i++];
	if (dstsize)
		*dst = 0;
	return (ft_strlen(src));
}
