## Installation

### 1. Install Rust via [rustup](https://rustup.rs)

#### Linux / macOS / WSL

Open a terminal and run:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## **Usage**
The main command to run our tool is
```
cargo run <benchmark_file_or_dir> <cost_file>
```
- `benchmark_file_or_dir` will run the tool on either a directory containing benchmarks or a single benchmark file.
- `cost_file` is expected to be a JSON file defining the cost model for optimization. By default, we provide `costs.json` which uses the cost model as defined in the final project description.

Running `cargo run benchmarks costs.json` will display all of our key results.

## **Results**
The results of the benchmarks are saved in `benchmark_output.txt`. Each entry includes the original expression, its cost, the optimized expression, and the corresponding optimized cost.