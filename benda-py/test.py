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
def gen_tree(depth, n):
    return gen_tree(depth-1, n-1)


print(gen_tree(1,2))