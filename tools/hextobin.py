#!/usr/bin/python
import sys

hexs = sys.argv[1:]

for h in hexs:
    i = int(h, 16)
    print "{0:08b}".format(i),
