from benda import bjit
import benda

@bjit
def sum_nums(a, b, mul):
    d = benda.switch()
    match a == b:
        case 0:
            return a + b
        case 1:
            return a * mul
    return d

print(sum_nums)