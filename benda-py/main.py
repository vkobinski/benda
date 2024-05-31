from benda import bjit
#import benda

#def test():
    #number = benda.u24(3)
    #number = number - benda.u24(2)
    #return number

def test():
    number = 3
    number = number - 2
    return number

@bjit
def sum_nums(a, b, mul):
    c = (a + b) * mul
    d = (c + b) / 2
    return d

print(sum_nums)