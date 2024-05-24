import benda
import ast, inspect

def test():
    number = benda.u24(3)
    number = number - benda.u24(2)
    return number

def test_ast():
    return 4

def print_ast():
    my_ast = ast.dump(ast.parse(inspect.getsource(test_ast)))
    return my_ast