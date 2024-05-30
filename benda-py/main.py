from benda import bjit
import benda

def test():
    number = benda.u24(3)
    number = number - benda.u24(2)
    return number

@bjit
def sum_nums():
    a = 5
    b = 2
    c = (a + b) * 4
    d = (c + b) / 2
    return d

print(sum_nums)