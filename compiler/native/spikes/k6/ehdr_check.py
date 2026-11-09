#!/usr/bin/env python3
"""K6: compare .eh_frame FDE count with the .eh_frame_hdr search table.

Usage: ehdr_check.py <exe> [<exe>...]
Prints FDEs in .eh_frame, the fde_count stored in the header, and the count
of overlapping pc ranges. A linker product is sound only if the first two
numbers match and overlaps are zero.
"""
import re
import struct
import subprocess
import sys


def fdes(path):
    out = subprocess.run(["readelf", "-wF", path], capture_output=True, text=True).stdout
    return [(int(a, 16), int(b, 16)) for a, b in re.findall(r"pc=([0-9a-f]+)\.\.([0-9a-f]+)", out)]


def hdr(path):
    out = subprocess.run(["readelf", "-x", ".eh_frame_hdr", path], capture_output=True, text=True).stdout
    hexs = []
    for line in out.splitlines():
        parts = line.split()
        if len(parts) > 2 and parts[0].startswith("0x"):
            hexs += parts[1:5]
    data = b"".join(bytes.fromhex(h) for h in hexs if len(h) == 8)
    if len(data) < 12:
        return None
    _, count = struct.unpack_from("<iI", data, 4)
    return count


for p in sys.argv[1:]:
    r = sorted(fdes(p))
    over = sum(1 for i in range(1, len(r)) if r[i][0] < r[i - 1][1])
    print(f"EHDR {p} fdes_in_eh_frame={len(r)} fde_count_in_hdr={hdr(p)} overlaps={over}")
