from benda import bjit
import benda

@bjit
def sum_nums(a, b, mul):
    d = benda.switch()
    match a:
        case 1:
            return a + b
        case 2:
            return a * mul
    return d

print(sum_nums)