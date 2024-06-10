type Leaf = (u24 value)
type Node = (Tree left) | (Tree right)

type Tree = Leaf | Node


def gen_tree(tree):
  match tree:
    case Leaf:
      return tree.value
    case Node:
      return tree.left + tree.right


def main:
  return gen_tree(10, 2)
