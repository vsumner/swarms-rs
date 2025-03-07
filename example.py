import swarms_rust

def my_callable():
    return "hello"

print(swarms_rust.run_callable_concurrently(my_callable, 5, None))