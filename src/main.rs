use egg::{*, rewrite as rw};
use std::fs;
use std::path::Path;

define_language! {
    pub enum FpExpr {
        "Fp2" = Fp2([Id; 2]),
        "Fp3" = Fp3([Id; 3]),
        "Fp4" = Fp4([Id; 2]),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "-" = Sub([Id; 2]),
        "constmul" = ConstMul([Id; 2]),
        "square" = Square(Id),
        "ξ" = Xi,
        "+2" = AddFp2([Id; 2]), // Fp2
        "*2" = MulFp2([Id; 2]),
        "-2" = SubFp2([Id; 2]),
        "constmul2" = ConstMulFp2([Id; 2]),
        "square2" = SquareFp2(Id),
        Const(u64),
        Symbol(Symbol),
    }
}

struct UnitCost;

impl CostFunction<FpExpr> for UnitCost {
    type Cost = f64;

    fn cost<C>(&mut self, enode: &FpExpr, mut costs: C) -> f64
    where
        C: FnMut(Id) -> f64,
    {
        let base_cost = match enode {
            // Fp
            FpExpr::Add(_) => 0.1,
            FpExpr::Sub(_) => 0.1,
            FpExpr::Mul(_) => 1.5,
            FpExpr::ConstMul(_) => 0.8,
            FpExpr::Square(_) => 1.0,

            // Fp2
            FpExpr::AddFp2(_) => 1.0,
            FpExpr::SubFp2(_) => 1.0,
            FpExpr::MulFp2(_) => 10.0,
            FpExpr::ConstMulFp2(_) => 4.0,
            FpExpr::SquareFp2(_) => 6.0,

            FpExpr::Symbol(_) => 0.0,
            FpExpr::Const(_) => 0.0,
            _ => 0.0,
        };
        base_cost + enode.children().iter().map(|&id| costs(id)).sum::<f64>()
    }
}

/*
struct UnitCost;

impl CostFunction<FpExpr> for UnitCost {
    type Cost = f64;

    fn cost<C>(&mut self, enode: &FpExpr, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        let base_cost = match enode {
            // Fp operations
            FpExpr::Add(_) => 0.1,
            FpExpr::Sub(_) => 0.1,
            FpExpr::Mul(_) => 1.5,
            FpExpr::Square(_) => 1.0,
            FpExpr::ConstMul(_) => 0.8,
            
            // Fp2 operations
            FpExpr::Fp2(_) => 0.0, // Constructor has no cost
            _ if enode.children().len() == 2 => {
                // Assume operations on Fp2 are more expensive
                match enode {
                    FpExpr::Add(_) => 1.0,
                    FpExpr::Sub(_) => 1.0,
                    FpExpr::Mul(_) => 10.0,
                    FpExpr::ConstMul(_) => 4.0,
                    FpExpr::Square(_) => 6.0,
                    _ => 0.0,
                }
            },
            
            FpExpr::Const(_) => 0.0,
            FpExpr::Symbol(_) => 0.0,
            FpExpr::Xi => 0.0,
            _ => 0.0,
        };
        
        base_cost + enode.children().iter().map(|&id| costs(id)).sum::<f64>()
    }
}
*/

fn is_const(a: &str, b: &str) -> impl Fn(&mut EGraph<FpExpr, ()>, Id, &Subst) -> bool {
    let a = a.parse().unwrap();
    let b = b.parse().unwrap();
    move |egraph, _, subst| {
        let is_const_node = |id: Id| {
            egraph[id].nodes.iter().any(|n| {
                matches!(n, FpExpr::Const(_)) || matches!(n, FpExpr::Xi)
            })
        };
        is_const_node(subst[a]) || is_const_node(subst[b])
    }
}

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
    let rules: &[Rewrite<FpExpr, ()>] = &[
        rw!("square_add"; "(square (+ ?a ?b))" => "(+ (+ (square ?a) (square ?b)) (* 2 (* ?a ?b)))"),
        rw!("mul_const"; "(* 2 ?a)" => "(+ ?a ?a)"),
        rw!("mul_const2"; "(constmul 2 ?a)" => "(+ ?a ?a)"),
        rw!("mulsquare"; "(* ?x ?x)" => "(square ?x)"),
        rw!("mul_const_fp2"; "(*2 2 ?a)" => "(+2 ?a ?a)"),
        rw!("mul_const2_fp2"; "(constmul2 2 ?a)" => "(+2 ?a ?a)"),
        rw!("mulsquare_fp2"; "(*2 ?x ?x)" => "(square2 ?x)"),
        rw!(
            "two_xy_to_squares";
            "(* 2 (* ?x ?y))" => "(- (square (+ ?x ?y)) (+ (square ?x) (square ?y)))" // 2xy = (x+y)^2 - x^2 - y^2
        ),
        rw!("mul_to_constmul"; "(* ?a ?b)" => "(constmul ?a ?b)" if is_const("?a", "?b")),

        rw!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rw!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rw!("assoc-add"; "(+ (+ ?a ?b) ?c)" => "(+ ?a (+ ?b ?c))"),
        rw!("assoc-mul"; "(* (* ?a ?b) ?c)" => "(* ?a (* ?b ?c))"),
    ];

    for (name, expr) in benchmarks {
        println!("\n=== Benchmark: {} ===", name);

        let runner = Runner::default()
            .with_expr(&expr)
            .run(rules);

        let extractor = Extractor::new(&runner.egraph, UnitCost);
        let (best_cost, best_expr) = extractor.find_best(runner.roots[0]);

        println!("Original expr: {}", expr);
        println!("Optimized expr: {}", best_expr);
        println!("Cost: {}", best_cost);
    }
}