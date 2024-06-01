from dataclasses import dataclass
from benda import bjit, u24
import benda

@dataclass
class Leaf:
    value: u24

@bjit
def sum_nums(a):

    leaf = Leaf(a)

    #d = benda.switch()
    #match a == b:
    #    case 0:
    #        return a + b
    #    case 1:
    #        return a * mul
    #return d

    return leaf

print(sum_nums)