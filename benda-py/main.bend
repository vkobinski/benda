type Leaf = u24
type Node = (Tree left) | (Tree right)

type Tree = Leaf | Node

def main:
  tree = Tree/Leaf(2)
  o_tree = Tree/Leaf(2)
  return Tree/Node(tree, o_tree)