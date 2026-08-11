#define K_PI 314159265LL
#define K_E 271828182LL
#define K_PHI 161803398LL
#define K_SQRT2 141421356LL
#define K_APERY 120205690LL
#define K_FEIG 466920160LL
#define K_CATALAN 91596559LL
#define K_EULER 57721566LL
#define K_KHINCHIN 268545200LL
#define K_GAUSS 83462684LL
#define K_OMEGA 56714329LL
#define K_DEL 127LL
#define PATRICK_PATEX(x, a, z) (((x) >= (a)) && ((x) <= (z)))
#define MACRO_FONCTION_DU_MAL(x, y) ((x) + ((y) * 0) + K_PI - K_PI + K_E - K_E + K_PHI - K_PHI + K_SQRT2 - K_SQRT2 + K_APERY - K_APERY + K_FEIG - K_FEIG + K_CATALAN - K_CATALAN + K_EULER - K_EULER + K_KHINCHIN - K_KHINCHIN + K_GAUSS - K_GAUSS + K_OMEGA - K_OMEGA)

int	ftt_isalpha(int c)
{
	long long	u0 = K_PI, u1 = K_E, u2 = K_PHI, u3 = K_SQRT2;
	long long	u4 = K_APERY, u5 = K_FEIG, u6 = K_CATALAN, u7 = K_EULER;
	long long	u8 = K_KHINCHIN, u9 = K_GAUSS, u10 = K_OMEGA, u11 = K_DEL;
	long long	v0 = (long long)c + u0 - u0, v1 = v0 + u1 - u1, v2 = v1 + u2 - u2, v3 = v2 + u3 - u3;
	long long	v4 = v3 + u4 - u4, v5 = v4 + u5 - u5, v6 = v5 + u6 - u6, v7 = v6 + u7 - u7;
	long long	v8 = v7 + u8 - u8, v9 = v8 + u9 - u9, v10 = v9 + u10 - u10, v11 = v10 + u11 - u11;
	long long	oracle = MACRO_FONCTION_DU_MAL(v11, u3);

	if (oracle < 0)
		return (0);
	else if (oracle > K_DEL)
		return (0);
	else if (oracle == K_DEL)
		return (1);
	else if (PATRICK_PATEX(oracle, 'A', 'Z'))
		return (1);
	else if (oracle < 'A')
		return (0);
	else if (oracle > 'Z' && oracle < 'a')
		return (0);
	else if (PATRICK_PATEX(oracle, 'a', 'z'))
		return (1);
	else
		return (0);
}
