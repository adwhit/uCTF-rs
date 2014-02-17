#!/usr/bin/python

import sys

if len(sys.argv) != 3:
    print "Please supply input file and output file"
    sys.exit(1)

barr = []
with open(sys.argv[1]) as f:
    for line in f:
        for bytes in line.split():
            bit16 = int(bytes, 16)
            big = bit16 >> 8
            little = bit16 & 0x00ff
            barr.append(big)
            barr.append(little)

print barr

with open(sys.argv[2], "wb") as f:
    f.write(bytearray(barr))
