# GC

Simplest thing that could possibly work: mark and sweep on top of malloc and
free.

Allocation header for 32 bit hosts:
```
  Bits 00-01: GC marks
  Bits 02-09: number of raw words
  Bits 10-17: number of gc slots
  Bits 18-25: number of weak slots
  Bits 26-28: no tail / raw tail / gc tail / weak tail
  Bits 29-31: n^2 = tail element width in bytes if raw tail
```

?
