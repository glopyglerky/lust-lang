// K3/K6: minimal libtcc API driver with the same call sequence as
// src/tcc.rs (tcc_new, set_error_func, set_lib_path, set_options "-w",
// set_output_type, compile_string/add_file/add_library, output_file).
//
// Usage: libtcc_driver <tcc_lib_path> <exe|obj|dll|run> <out> [opts] files...
//   With type "run" (out is "-"), the driver uses TCC_OUTPUT_MEMORY,
//   relocates, looks up "main", calls it with argc=0, and exits with its
//   return value. That mirrors run_in_memory in src/tcc.rs, minus the fork.
//   file ending in .cstr  -> read file, tcc_compile_string its contents
//   arg "-l:<dir>:<name>" -> tcc_add_library_path + tcc_add_library
//   arg "-o:<opts>"       -> extra tcc_set_options (e.g. -Wl,-rpath=...)
//   anything else         -> tcc_add_file
// Exit 0 on success. Prints libtcc errors to stderr.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "libtcc.h"

static void on_error(void *opaque, const char *msg) {
    (void)opaque;
    fprintf(stderr, "libtcc: %s\n", msg);
}
static char *slurp(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) return 0;
    fseek(f, 0, SEEK_END); long n = ftell(f); fseek(f, 0, SEEK_SET);
    char *buf = malloc((size_t)n + 1);
    if (fread(buf, 1, (size_t)n, f) != (size_t)n) { fclose(f); free(buf); return 0; }
    buf[n] = 0; fclose(f); return buf;
}
int main(int argc, char **argv) {
    if (argc < 5) { fprintf(stderr, "usage: libtcc_driver <lib_path> <exe|obj|dll> <out> files...\n"); return 2; }
    TCCState *s = tcc_new();
    if (!s) { fprintf(stderr, "tcc_new failed\n"); return 1; }
    tcc_set_error_func(s, 0, on_error);
    tcc_set_lib_path(s, argv[1]);
    tcc_set_options(s, "-w");
    int run_mode = strcmp(argv[2], "run") == 0;
    int ty = strcmp(argv[2], "obj") == 0 ? TCC_OUTPUT_OBJ
           : strcmp(argv[2], "dll") == 0 ? TCC_OUTPUT_DLL
           : run_mode ? TCC_OUTPUT_MEMORY : TCC_OUTPUT_EXE;
    tcc_set_output_type(s, ty);
    for (int i = 4; i < argc; i++) {
        const char *a = argv[i];
        if (strncmp(a, "-l:", 3) == 0) {
            const char *sep = strrchr(a + 3, ':');
            char dir[1024]; size_t n = (size_t)(sep - (a + 3));
            memcpy(dir, a + 3, n); dir[n] = 0;
            tcc_add_library_path(s, dir);
            if (tcc_add_library(s, sep + 1) != 0) { fprintf(stderr, "add_library %s failed\n", sep + 1); return 1; }
        } else if (strncmp(a, "-o:", 3) == 0) {
            tcc_set_options(s, a + 3);
        } else if (strlen(a) > 5 && strcmp(a + strlen(a) - 5, ".cstr") == 0) {
            char *src = slurp(a);
            if (!src || tcc_compile_string(s, src) != 0) { fprintf(stderr, "compile_string %s failed\n", a); return 1; }
            free(src);
        } else if (tcc_add_file(s, a) != 0) {
            fprintf(stderr, "add_file %s failed\n", a); return 1;
        }
    }
    if (run_mode) {
        if (tcc_relocate(s) != 0) { fprintf(stderr, "relocate failed\n"); return 1; }
        void *sym = tcc_get_symbol(s, "main");
        if (!sym) { fprintf(stderr, "get_symbol main failed\n"); return 1; }
        int (*entry)(int, char **) = (int (*)(int, char **))sym;
        char *args[1] = {(char *)"probe"};
        int rc = entry(1, args);
        printf("INFO libtcc_run_memory main_returned=%d\n", rc);
        return rc;
    }
    if (tcc_output_file(s, argv[3]) != 0) { fprintf(stderr, "output_file %s failed\n", argv[3]); return 1; }
    tcc_delete(s);
    return 0;
}
