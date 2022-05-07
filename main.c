#include<stdio.h>

extern int start();
extern int duplicate(int);

int main() {
    int value = start();
    value = duplicate(value);
    printf("%d\n", value);
    return 0;
}
