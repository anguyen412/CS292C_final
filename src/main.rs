use egg::{*, rewrite as rw};
use std::fs;
use std::path::Path;
use std::env::args;

mod dsl;
use dsl::{*};

mod analysis;
use analysis::{*};

fn load_benchmark(file: &str) -> (String, RecExpr<FpExpr>) {
    let content = fs::read_to_string(file).unwrap();
    let parsed = content.parse::<RecExpr<FpExpr>>().unwrap_or_else(|e| {
        panic!("Failed to parse file {}: {}", file, e);
    });
    let name = Path::new(file).file_name().unwrap().to_string_lossy().into_owned();
    (name, parsed)
}

fn load_benchmarks(dir: &str) -> Vec<(String, RecExpr<FpExpr>)> {
    let mut benchmarks = Vec::new();
    if Path::new(dir).is_file() {
        // If the path is a file, load it directly
        let (name, expr) = load_benchmark(dir);
        benchmarks.push((name, expr));
        return benchmarks;
    }

    let mut entries: Vec<_> = fs::read_dir(Path::new(dir))
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect();

    // sort alphabetically by file name
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let (name, expr) = load_benchmark(&entry.path().to_string_lossy());
        benchmarks.push((name, expr));
    }
    benchmarks
}

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 3 {
        eprintln!("Usage: {} <benchmark_file_or_dir> <cost_file>", args[0]);
        return;
    }
    let benchmarks_dir = &args[1];
    let cost_file = &args[2];
    if !Path::new(benchmarks_dir).exists() {
        eprintln!("Benchmark file or directory does not exist: {}", benchmarks_dir);
        return;
    }
    if !Path::new(cost_file).exists() {
        eprintln!("Cost file does not exist: {}", cost_file);
        return;
    }

    let benchmarks = load_benchmarks(benchmarks_dir);
    let rules: &[Rewrite<FpExpr, ConstantFolding>] = &[
        rw!("square_add"; "(square (+ ?a ?b))" => "(+ (+ (square ?a) (square ?b)) (* 2 (* ?a ?b)))"),
        rw!("mul_const"; "(* 2 ?a)" => "(+ ?a ?a)"),
        rw!("mul_const2"; "(constmul 2 ?a)" => "(+ ?a ?a)"),
        rw!("add_const0"; "(+ 0 ?a)" => "?a"),
        rw!("mul_const0"; "(constmul 0 ?a)" => "0"),
        rw!("mul_const1"; "(constmul 1 ?a)" => "?a"),
        rw!("mulsquare"; "(* ?x ?x)" => "(square ?x)"),
        rw!(
            "two_xy_to_squares";
            "(* 2 (* ?x ?y))" => "(- (square (+ ?x ?y)) (+ (square ?x) (square ?y)))" // 2xy = (x+y)^2 - x^2 - y^2
        ),
        rw!("mul_to_constmul"; "(* ?a ?b)" => "(constmul ?a ?b)" if is_const("?a", "?b")),

        rw!("sub-self"; "(- ?a ?a)" => "0"),
        rw!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rw!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rw!("assoc-add"; "(+ (+ ?a ?b) ?c)" => "(+ ?a (+ ?b ?c))"),
        rw!("assoc-mul"; "(* (* ?a ?b) ?c)" => "(* ?a (* ?b ?c))"),
        rw!("distribute-mul"; "(* ?a (+ ?b ?c))" => "(+ (* ?a ?b) (* ?a ?c))"),
        rw!("factor-out"; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
        rw!("double-inverse"; "(inv (inv ?a))" => "?a"),

        // Fp2 Rules
        rw!("mul_const_fp2"; "(*2 2 ?a)" => "(+2 ?a ?a)"),
        rw!("mul_const2_fp2"; "(constmul2 2 ?a)" => "(+2 ?a ?a)"),
        rw!("mulsquare_fp2"; "(*2 ?x ?x)" => "(square2 ?x)"),
        rw!("mul_to_constmul_fp2"; "(*2 ?a ?b)" => "(constmul2 ?a ?b)" if is_const("?a", "?b")),
        rw!("commute-add_fp2"; "(+2 ?a ?b)" => "(+2 ?b ?a)"),
        rw!("commute-mul_fp2"; "(*2 ?a ?b)" => "(*2 ?b ?a)"),
        rw!("assoc-add_fp2"; "(+2 (+2 ?a ?b) ?c)" => "(+2 ?a (+2 ?b ?c))"),
        rw!("assoc-mul_fp2"; "(*2 (*2 ?a ?b) ?c)" => "(*2 ?a (*2 ?b ?c))"),
        rw!("distribute-mul_fp2"; "(*2 ?a (+2 ?b ?c))" => "(+2 (*2 ?a ?b) (*2 ?a ?c))"),
        rw!("factor-out_fp2"; "(+2 (*2 ?a ?b) (*2 ?a ?c))" => "(*2 ?a (+2 ?b ?c))"),

        // Addition in Fp2 (Alg 5)
        rw!("add_fp2"; "(+2 (Fp2 ?a0 ?a1) (Fp2 ?b0 ?b1))" => "(Fp2 (+ ?a0 ?b0) (+ ?a1 ?b1))"),
        // Subtraction in Fp2 (Alg 6)
        rw!("sub_fp2"; "(-2 (Fp2 ?a0 ?a1) (Fp2 ?b0 ?b1))" => "(Fp2 (- ?a0 ?b0) (- ?a1 ?b1))"),
        // Subtraction in Fp2 (Alg 7)
        rw!("mul_fp2"; "(*2 (Fp2 ?a0 ?a1) ?b0)" => "(Fp2 (* ?a0 ?b0) (* ?a1 ?b0))"),
        // Inverse in Fp2 (Alg 8)
        rw!("inv_fp2_short";
            "(inv2 (Fp2 ?x ?y))" 
            => 
            "(Fp2 
                (* ?x (inv (+ (square ?x) (square ?y)))) 
                (* (- 0 ?y) (inv (+ (square ?x) (square ?y))))
            )"
        ),
        rw!("double-inverse_fp2"; "(inv2 (inv2 ?a))" => "?a"),
        rw!(
            "two_xy_to_squares_fp2";
            "(*2 2 (*2 ?x ?y))" => "(-2 (square2 (+2 ?x ?y)) (+2 (square2 ?x) (square2 ?y)))" // 2xy = (x+y)^2 - x^2 - y^2
        ),

        // Addition in Fp6 (Alg 10)
        rw!("add_fp6"; "(+6 (Fp6 ?a0 ?a1 ?a2) (Fp6 ?b0 ?b1 ?b2))" => "(Fp6 (+2 ?a0 ?b0) (+2 ?a1 ?b1) (+2 ?a2 ?b2))"),
        // Subtraction in Fp6 (Alg 11)
        rw!("sub_fp6"; "(-6 (Fp6 ?a0 ?a1 ?a2) (Fp6 ?b0 ?b1 ?b2))" => "(Fp6 (-2 ?a0 ?b0) (-2 ?a1 ?b1) (-2 ?a2 ?b2))"),
    ];

    for (name, expr) in benchmarks {
        println!("\n=== Benchmark: {} ===", name);

        // Compute original cost before applying rules
        let runner = Runner::<FpExpr, ()>::default().with_expr(&expr);
        let cf = FpCost::new(cost_file);
        let extractor = Extractor::new(&runner.egraph, cf);
        let (original_cost, _) = extractor.find_best(runner.roots[0]);

        let runner = Runner::<FpExpr, ConstantFolding, ()>::default()
            .with_expr(&expr)
            .run(rules);

        let cf = FpCost::new(cost_file);
        let extractor = Extractor::new(&runner.egraph, cf);
        let (_, best_expr) = extractor.find_best(runner.roots[0]);
        let cf = FpCost::new(cost_file);
        let (best_cost, cse_expr) = extract_common_subexpressions(&best_expr, cf);

        println!("Original expr: {}", expr);
        println!("Original cost: {}", original_cost);
        println!("Optimized expr:\n{}", cse_expr);
        println!("Optimized cost: {}", best_cost);
    }
}