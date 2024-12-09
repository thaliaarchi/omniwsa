#include <ctype.h>
#include <limits.h>
#include <stdio.h>

#define CHECK(c, is) if (is(c)) { printf(" " #is); }

int main() {
    for (int c = 0; c <= UCHAR_MAX; c++) {
        printf("0x%02X", c);
        CHECK(c, isalnum);
        CHECK(c, isalpha);
        CHECK(c, isblank);
        CHECK(c, iscntrl);
        CHECK(c, isdigit);
        CHECK(c, isgraph);
        CHECK(c, islower);
        CHECK(c, isprint);
        CHECK(c, ispunct);
        CHECK(c, isspace);
        CHECK(c, isupper);
        CHECK(c, isxdigit);
        printf("\n");
    }
    return 0;
}
