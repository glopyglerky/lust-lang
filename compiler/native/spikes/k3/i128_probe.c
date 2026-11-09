// K3: __int128 must fail under tcc. The runner treats a compile error as PASS.
#include <stdio.h>
int main(void) { __int128 x = ((__int128)1) << 90; printf("%llu\n", (unsigned long long)(x >> 89)); return 0; }
