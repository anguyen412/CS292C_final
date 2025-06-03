use egg::{*, rewrite as rw};
use std::fs;
use std::path::Path;

mod dsl;
use dsl::{*};

mod analysis;
use analysis::{*};

fn load_benchmarks(dir: &str) -> Vec<(String, RecExpr<FpExpr>)> {
    let mut benchmarks = Vec::new();
    for entry in fs::read_dir(Path::new(dir)).unwrap() {
        let entry = entry.unwrap();
        let content = fs::read_to_string(entry.path()).unwrap();
        let parsed = content.parse::<RecExpr<FpExpr>>().unwrap_or_else(|e| {
            panic!("Failed to parse file {}: {}", entry.path().display(), e);
        });
        benchmarks.push((
            entry.file_name().into_string().unwrap(),
            parsed
        ));
    }
    benchmarks
}

fn main() {
    let benchmarks = load_benchmarks("benchmarks");
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
        let cf = FpCost::new("costs.json");
        let extractor = Extractor::new(&runner.egraph, cf);
        let (original_cost, _) = extractor.find_best(runner.roots[0]);

        let runner = Runner::<FpExpr, ConstantFolding, ()>::default()
            .with_expr(&expr)
            .run(rules);

        let cf = FpCost::new("costs.json");
        let extractor = Extractor::new(&runner.egraph, cf);
        let (_, best_expr) = extractor.find_best(runner.roots[0]);
        let (best_cost, cse_expr) = extract_common_subexpressions(&best_expr, FpCost::new("costs.json"));

        println!("Original expr: {}", expr);
        println!("Original cost: {}", original_cost);
        println!("Optimized expr:\n{}", cse_expr);
        println!("Optimized cost: {}", best_cost);
    }
}