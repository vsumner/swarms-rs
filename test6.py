import multiprocessing

def worker(x):
    # Example CPU-bound operation: squaring the number
    return x * x

if __name__ == '__main__':
    # Create a pool with 4 processes
    with multiprocessing.Pool(processes=4) as pool:
        results = pool.map(worker, range(10))
    print("Multiprocessing Pool results:", results)
