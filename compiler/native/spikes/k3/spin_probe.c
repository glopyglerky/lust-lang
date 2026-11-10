// K3: spin_loop hint forms. Build with -DSPIN_FORM=<n>:
// 1 __builtin_ia32_pause()      2 _mm_pause via xmmintrin.h
// 3 __asm__ volatile("pause")   4 __asm__ volatile("rep nop")
// 5 atomic_signal_fence (plain compiler barrier)   6 __builtin_arm_yield()
#ifndef SPIN_FORM
#define SPIN_FORM 3
#endif
#include <stdio.h>
#if SPIN_FORM == 2
#include <xmmintrin.h>
#endif
#if SPIN_FORM == 5
#include <stdatomic.h>
#endif
int main(void) {
#if SPIN_FORM == 1
    __builtin_ia32_pause();
#elif SPIN_FORM == 2
    _mm_pause();
#elif SPIN_FORM == 3
    __asm__ volatile("pause");
#elif SPIN_FORM == 4
    __asm__ volatile("rep nop");
#elif SPIN_FORM == 5
    atomic_signal_fence(memory_order_seq_cst);
#elif SPIN_FORM == 6
    __builtin_arm_yield();
#endif
    printf("PASS spin_form_%d\nRESULT PASS fails=0\n", SPIN_FORM);
    return 0;
}
