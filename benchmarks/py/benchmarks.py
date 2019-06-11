
import time

def benchmark(name, f, args):
    t0 = time.clock()
    f(*args)
    t1 = time.clock()
    print("{}: {}".format(name, t1-t0))

def sumFloats(floats):
    sum = 0;
    for f in floats:
        sum += f
    return sum

def doSumFloats():
    arr = []
    for i in range(1, 150000):
        arr.append(float(i))
    benchmark("SumFloats", sumFloats, [arr])

def fibonacci(i):
    if i < 2:
        return 1
    else:
        return fibonacci(i - 1) + fibonacci(i - 2)

def doFibonacci():
    benchmark("Fibonacci", fibonacci, [21])

if __name__ == "__main__":
    doSumFloats()
    doFibonacci()
