import struct
from collections import Counter

densities = Counter()

with open("puzzles", "rb") as f:
    i = -1
    while True:
        print(i, f.tell())
        i += 1
        n = f.read(4)
        if not n:
            break
        width, = struct.unpack("f", n)
        height, = struct.unpack("f", f.read(4))
        density, = struct.unpack("f", f.read(4))
        densities[density] += 1
        print(width, height)
        f.read(4*int(width*height))

print(densities.most_common())
