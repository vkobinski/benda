
type Leaf = u24
type Tree = (Node ~left ~right) | Leaf

def main:
  return Tree/Leaf(1)