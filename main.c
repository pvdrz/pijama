#include<stdio.h>

extern int start();

int main() {
    int value = start();
    printf("%d\n", value);
    return 0;
}
