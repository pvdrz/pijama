#include <stdio.h>

extern long start();
extern long duplicate(long);

int main() {
  long value = start();
  value = duplicate(value);
  printf("%ld\n", value);

  return 0;
}
