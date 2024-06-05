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
def sum_tree(depth, n):
    tree = gen_tree(depth, n)
    match tree:
        case Leaf(value):
            return value
        case Node(left, right):
            return sum_tree(left) + sum_tree(right)
       
def gen_tree(depth, n):
    if depth == 0:
        return Leaf(n)
    else:
        return Node(gen_tree(depth-1, n-1), gen_tree(depth-1, n+1))

print(sum_tree(2,5))