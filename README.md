## **Installation**
1. Install Rust via Rustup
2. Build the tool using
    ```
    cargo build
    ```

## **Usage**
The main command to run our tool is
```
cargo run <benchmark_file_or_dir> <cost_file>
```
- `benchmark_file_or_dir` will run the tool on either a directory containing benchmarks or a single benchmark file.
- `cost_file` is expected to be a JSON file defining the cost model for optimization. By default, we provide `costs.json` which uses the cost model as defined in the final project description.
Running `cargo run benchmarks costs.json` will display all of our key results.