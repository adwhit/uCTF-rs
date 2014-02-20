#!/usr/bin/python
import sys

bins = sys.argv[1:]

for b in bins:
    i = int(b, 2)
    print "{0:02x}".format(i),
