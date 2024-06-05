type Leaf = u24
type Node = (Tree left) | (Tree right)

type Tree = Leaf | Node

def gen_tree(depth, n):
  if depth == 0:
    return Leaf/u24(n)
  else:
    return Node(gen_tree(depth - 1, n - 1), gen_tree(depth - 1, n + 1))

def main:
  return gen_tree(10,2)