from dataclasses import dataclass
from benda import bjit, u24
import benda

@dataclass
class Leaf:
    value: u24

@dataclass
class Node:
    left: 'Tree'
    right: 'Tree'

Tree = Node | Leaf

@bjit
def sum_nums(a):

    node = Tree(Node(Leaf(a), Leaf(2)))

    #d = benda.switch()
    #match a == b:
    #    case 0:
    #        return a + b
    #    case 1:
    #        return a * mul
    #return d

    return node

print(sum_nums)