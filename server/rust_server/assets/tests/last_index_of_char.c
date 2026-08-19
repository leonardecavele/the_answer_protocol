extern int last_index_of_char(char *str, char c);

int main(void)
{
    if (last_index_of_char((void *)0, 'a') != -1 || last_index_of_char("", 'a') != -1)
        return 0;
    if (last_index_of_char("banana", 'a') != 5 || last_index_of_char("banana", 'n') != 4)
        return 0;
    if (last_index_of_char("banana", 'b') != 0 || last_index_of_char("banana", 'z') != -1)
        return 0;
    if (last_index_of_char("banana", '\0') != -1)
        return 0;
    return SANDBOX_SUCCESS;
}
