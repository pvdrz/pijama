#include <stdio.h>
#include <stdint.h>

extern int64_t sum(int64_t);

void main() {
    printf("%d\n", sum(100));
}
