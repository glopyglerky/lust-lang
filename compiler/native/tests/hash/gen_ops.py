#!/usr/bin/env python3
"""Generates deterministic operation streams for the hash difttest.

Both the Rust oracle and the C harness read the same stream and print
state after every step, so a byte diff of the outputs is the gate.
Usage: gen_ops.py <map|set|index> <seed> <num_ops>
Keys and values are plain integers; each program maps them to its key
type (see README.md). The PRNG is splitmix64, so streams do not depend
on the Python version.

Phases: mostly insert/remove churn on a small key space (many replaces
and tombstones), with periodic reserve/shrink/clear/retain/drain/reset
and remove storms that drop most keys then refill (which is what forces
hashbrown's rehash-in-place and resize paths).
"""
import sys

MASK = (1 << 64) - 1


class Rng:
    def __init__(self, seed):
        self.s = seed & MASK

    def next(self):
        self.s = (self.s + 0x9E3779B97F4A7C15) & MASK
        z = self.s
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
        return z ^ (z >> 31)

    def below(self, n):
        return self.next() % n


def main():
    mode, seed, n = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    r = Rng(seed ^ (0xB5297A4D9E3779B9 if mode != "map" else 0))
    out = sys.stdout
    out.write("n\n")
    keyspace = 240 if mode != "index" else 160
    live = 0  # rough guess, only used to bias index-drawing ops

    def key():
        # Mostly small keys, sometimes a large one for wide hash bits.
        if r.below(16) == 0:
            return r.next() % (1 << 40)
        return r.below(keyspace)

    for step in range(n):
        roll = r.below(1000)
        # Remove storm: every ~4000 steps, a burst that empties most of
        # the key space, then normal churn refills it.
        if step >= 4000 and step % 4000 < keyspace and step % 4000 > 0:
            k = step % 4000 - 1
            if k < keyspace:
                out.write(f"r {k}\n" if mode != "index" else f"s {k}\n")
                continue
        if mode == "index":
            if roll < 380:
                out.write(f"i {key()} {r.below(100000)}\n")
            elif roll < 430:
                out.write(f"u {key()} {r.below(100000)}\n")
            elif roll < 470:
                out.write(f"g {key()}\n")
            elif roll < 500:
                out.write(f"G {r.below(200)}\n")
            elif roll < 620:
                out.write(f"s {key()}\n")
            elif roll < 720:
                out.write(f"x {key()}\n")
            elif roll < 760:
                out.write(f"S {r.below(200)}\n")
            elif roll < 790:
                out.write(f"X {r.below(200)}\n")
            elif roll < 810:
                out.write("p\n")
            elif roll < 830:
                out.write(f"j {r.below(200)} {key()} {r.below(100000)}\n")
            elif roll < 850:
                out.write(f"R {r.below(300)}\n")
            elif roll < 860:
                out.write(f"E {r.below(60)}\n")
            elif roll < 875:
                out.write(f"h {r.below(120)}\n")
            elif roll < 885:
                out.write("F\n")
            elif roll < 900:
                out.write(f"T {2 + r.below(4)}\n")
            elif roll < 915:
                a = r.below(200)
                out.write(f"d {a} {a + r.below(80)}\n")
            elif roll < 925:
                out.write(f"t {r.below(200)}\n")
            elif roll < 935:
                out.write("v\n")
            elif roll < 950:
                out.write(f"m {r.below(200)} {r.below(200)}\n")
            elif roll < 960:
                out.write(f"W {r.below(200)} {r.below(200)}\n")
            elif roll < 975:
                out.write("c\n")
            elif roll < 988:
                out.write(f"w {r.below(64)}\n")
            else:
                out.write("n\n")
        else:
            if roll < 420:
                out.write(f"i {key()} {r.below(100000)}\n")
            elif roll < 480:
                out.write(f"e {key()} {r.below(100000)}\n")
            elif roll < 560:
                out.write(f"g {key()}\n")
            elif roll < 760:
                out.write(f"r {key()}\n")
            elif roll < 790:
                out.write(f"R {r.below(400)}\n")
            elif roll < 810:
                out.write(f"s {r.below(200)}\n")
            elif roll < 825:
                out.write("F\n")
            elif roll < 845:
                out.write(f"T {2 + r.below(4)}\n")
            elif roll < 860:
                out.write(f"x {2 + r.below(4)}\n")
            elif roll < 875:
                out.write(f"d {r.below(12)}\n")
            elif roll < 885:
                out.write("l\n")
            elif roll < 905:
                out.write("c\n")
            elif roll < 960:
                # fill burst: sequential inserts drive clean growth
                base = r.below(keyspace)
                for j in range(r.below(24)):
                    out.write(f"i {(base + j) % keyspace} {r.below(100000)}\n")
            elif roll < 985:
                out.write(f"w {r.below(80)}\n")
            else:
                out.write("n\n")
        live += 1


if __name__ == "__main__":
    main()
