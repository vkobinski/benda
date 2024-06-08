from dataclasses import dataclass
from benda import bjit, u24, bjit_test

@dataclass
class Leaf:
    value: u24

@dataclass
class Node:
    left: 'Tree'
    right: 'Tree'

Tree = Node | Leaf

def gen_tree(depth, n):
    if depth == 0:
        return Leaf(n)
    else:
        return Node(gen_tree(depth-1, n-1), gen_tree(depth-1, n+1))

@bjit
def sum_tree(tree: Tree):
    match tree:
       case Node(left, right):
            return sum_tree(left) + sum_tree(right)
       case Leaf(value):
            return value
        
       
tree = gen_tree(15, 10)
val = sum_tree(tree)
print("Somando a árvore:")
print(val)