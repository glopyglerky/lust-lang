// K6: allocator ownership probe, run as its own process. Frees a shim buffer
// with C free(). This is undefined behavior by contract. The probe only
// records what this platform does; a clean exit does not make it legal.
#include <stdio.h>
#include <stdlib.h>
#include "shim.h"
int main(void) {
    unsigned char *buf = shim_alloc_buffer(4096, 0xab);
    printf("INFO crossfree shim_ptr=%p check=%d\n", (void *)buf, shim_check_buffer(buf, 4096, 0xab));
    free(buf); // wrong owner on purpose
    printf("INFO crossfree survived C free of shim buffer\n");
    void *mine = malloc(4096);
    shim_free_buffer((unsigned char *)mine, 4096); // wrong owner on purpose
    printf("INFO crossfree survived shim free of C buffer\nRESULT OBSERVED\n");
    return 0;
}
