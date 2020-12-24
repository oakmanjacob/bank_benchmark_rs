# Bank Benchmark Example in Rust
by Jacob Oakman with help from Lachlan Sneff

## Purpose
This is an example meant to explore concurrency in rust through simulating a simple bank system by adapting a C assignment. The original implementation can be found [here](https://github.com/oakmanjacob/bank_benchmark_c).

## Explanation
We use a simple map implementation to model a list of accounts in a bank. A series of commands are run on seperate threads either transferring money between accounts or summing the total money across all accounts as a simple correctness check.

## Running
The software can be run with default parameters like this
```bash
$> cargo run --release
```
Parameters can also be configured
```bash
$> cargo run --release -- -h
    Finished release [optimized] target(s) in 0.05s
     Running `target\release\bank_benchmark.exe -h`
bank_benchmark 0.1.0

USAGE:
    bank_benchmark.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --account_count <account-count>     [default: 1024]
    -c, --command_count <command-count>     [default: 1024]
    -s, --stripe_count <stripe-count>       [default: 256]
    -t, --thread_count <thread-count>       [default: 1]
```
To run the benchmark with 4 threads and 4028 command operations you would run
```bash
$> cargo run --release -- -t 4 -c 2048
```

## Assignment Instructions
Here is a slightly adapted version of the origional instructions with c specific things removed

### Step 1
Define a map of types <int,double>. This map represents a collection of bank accounts:

- each account has a unique ID of type int 
- each account has an amount of fund of type double

### Step 2
Populate the entire map with the 'insert' function.  
Initialize the map in a way the sum of the amounts of all the accounts in the map is 100000  

### Step 3
Define a function "transfer" that selects two random bank accounts and an amount. This amount is subtracted from the amount of the first account and summed to the amount of the second account. The execution of the whole function should happen atomically: no operation should happen on B1 and B2 (or on the whole map?) while the function executes.  

### Step 4
Define a function "balance" that sums the amount of all the bank accounts in the map. In order to have a consistent result, the execution of this function should happen atomically: no other deposit operations should interleave.

### Step 5
Define a function 'do_work', which has a for-loop that iterates for config_t.iters times. In each iteration, the function 'transfer' should be called with 95% of the probability; otherwise (the rest 5%) the function 'balance' should be called.  
  
The function 'do_work' should measure 'exec_time_i', which is the time needed to perform the entire for-loop. This time will be shared with the main thread once the thread executing the 'do_work' joins its execution with the main thread.

### Step 6
Spread the total number of instructions between a specific number of threads and execute them concurrently. Collect the maximum execution time of any of the threads and use this as the execution time. After execution, call the balance function one final time.

- What is the output of this call of "balance"?
- Does it match what you expect?
- What do you expect?
- What are the outcomes of all the "balance" calls during the execution?
- Is that what you expect?

### Step 7
Now configure your application to perform the SAME TOTAL amount of iterations just executed, but all done by a single thread. Measure the time to perform them and compare with the time previously collected.

- Which conclusion can you draw?  
- Which optimization can you do to the single-threaded execution in order to improve its performance?    

### Final step: Produce plot
Each submission should include at least one plot in which the x-axis is the concurrent threads used {1;2;4;8} the y-axis is the application execution time. The performance at 1 thread must be the sequential application without atomic execution

## Results
Here are the graphs showing the execution time

### Test Platform
Intel Core i7-4790CPU @ 3.60GHz  
16GB DDR3 1600MHz  
Windows 10  

### Plots
![](plots/performance_debug_1024.png?raw=true)
![](plots/performance_debug_1048576.png?raw=true)
![](plots/performance_release_1024.png?raw=true)
![](plots/performance_release_1048576.png?raw=true)
