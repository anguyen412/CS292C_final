use egg::{*, rewrite as rw};
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

define_language! {
    pub enum FpExpr {
        "Fp2" = Fp2([Id; 2]),
        "Fp4" = Fp4([Id; 2]),
        "Fp6" = Fp6([Id; 3]),
        "ξ" = Xi,
        Const(u64),
        Symbol(Symbol),

        // Fp
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "-" = Sub([Id; 2]),
        "constmul" = ConstMul([Id; 2]),
        "square" = Square(Id),
        "inv" = Inv(Id),
        
        // Fp2
        "+2" = AddFp2([Id; 2]),
        "*2" = MulFp2([Id; 2]),
        "-2" = SubFp2([Id; 2]),
        "constmul2" = ConstMulFp2([Id; 2]),
        "square2" = SquareFp2(Id),
        "inv2" = InvFp2(Id),

        // Fp4
        "+4" = AddFp4([Id; 2]),
        "*4" = MulFp4([Id; 2]),
        "-4" = SubFp4([Id; 2]),
        "constmul4" = ConstMulFp4([Id; 2]),
        "square4" = SquareFp4(Id),

        // Fp6
        "+6" = AddFp6([Id; 2]),
        "*6" = MulFp6([Id; 2]),
        "-6" = SubFp6([Id; 2]),
        "constmul6" = ConstMulFp6([Id; 2]),
        "square6" = SquareFp6(Id),

        // Fp12
        "+12" = AddFp12([Id; 2]),
        "*12" = MulFp12([Id; 2]),
        "-12" = SubFp12([Id; 2]),
        "constmul12" = ConstMulFp12([Id; 2]),
        "square12" = SquareFp12(Id),
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
            FpExpr::Inv(_) => 20.0,

            // Fp2
            FpExpr::AddFp2(_) => 1.0,
            FpExpr::SubFp2(_) => 1.0,
            FpExpr::MulFp2(_) => 10.0,
            FpExpr::ConstMulFp2(_) => 4.0,
            FpExpr::SquareFp2(_) => 6.0,
            FpExpr::InvFp2(_) => 80.0,

            // Fp4
            FpExpr::AddFp4(_) => 3.0,
            FpExpr::SubFp4(_) => 3.0,
            FpExpr::MulFp4(_) => 40.0,
            FpExpr::ConstMulFp4(_) => 15.0,
            FpExpr::SquareFp4(_) => 27.0,

            // Fp6
            FpExpr::AddFp6(_) => 6.0,
            FpExpr::SubFp6(_) => 6.0,
            FpExpr::MulFp6(_) => 130.0,
            FpExpr::ConstMulFp6(_) => 50.0,
            FpExpr::SquareFp6(_) => 85.0,

            // Fp12
            FpExpr::AddFp12(_) => 12.0,
            FpExpr::SubFp12(_) => 12.0,
            FpExpr::MulFp12(_) => 310.0,
            FpExpr::ConstMulFp12(_) => 120.0,
            FpExpr::SquareFp12(_) => 200.0,

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

fn extract_common_subexpressions(expr: &RecExpr<FpExpr>) -> (f64, String) {
    let mut common_subexprs: HashMap<u64, Vec<Id>> = HashMap::new();
    let mut hash_cache: HashMap<Id, u64> = HashMap::new();
    let mut bindings: HashMap<u64, String> = HashMap::new();
    let mut let_defs = Vec::new();

    fn hash_subtree(expr: &RecExpr<FpExpr>, id: Id, cache: &mut HashMap<Id, u64>) -> u64 {
        if let Some(&h) = cache.get(&id) {
            return h;
        }
        let node = &expr[id];
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::mem::discriminant(node).hash(&mut hasher);
        match node {
            FpExpr::Const(val) => val.hash(&mut hasher),
            FpExpr::Symbol(sym) => sym.hash(&mut hasher),
            _ => {}
        }
        for &child in node.children() {
            let child_hash = hash_subtree(expr, child, cache);
            child_hash.hash(&mut hasher);
        }
        let h = hasher.finish();
        cache.insert(id, h);
        h
    }

    fn visit_subtree(expr: &RecExpr<FpExpr>, id: Id, cache: &mut HashMap<Id, u64>, hash_to_ids: &mut HashMap<u64, Vec<Id>>) {
        let node = &expr[id];
        for &child in node.children() {
            visit_subtree(expr, child, cache, hash_to_ids);
        }
        let h = hash_subtree(expr, id, cache);
        hash_to_ids.entry(h).or_default().push(id);
    }

    fn helper(expr: &RecExpr<FpExpr>, id: Id, common_subexprs: &HashMap<u64, Vec<Id>>, hash_cache: &mut HashMap<Id, u64>, bindings: &mut HashMap<u64, String>, let_defs: &mut Vec<(String, String)>) -> (f64, String) {
        let h = hash_subtree(expr, id, hash_cache);
        let node = &expr[id];

        if let Some(ids) = common_subexprs.get(&h) {
            if ids.len() > 1 && !matches!(node, FpExpr::Const(_) | FpExpr::Symbol(_) | FpExpr::Xi) {
                if let Some(name) = bindings.get(&h) {
                    return (0.0, name.clone());
                } else {
                    let (cost, s) = helper_children(expr, node, common_subexprs, hash_cache, bindings, let_defs);
                    let name = format!("t{}", let_defs.len());
                    bindings.insert(h, name.clone());
                    let_defs.push((name.clone(), s.clone()));
                    return (cost, name);
                }
            }
        }
        helper_children(expr, node, common_subexprs, hash_cache, bindings, let_defs)
    }

    fn helper_children(expr: &RecExpr<FpExpr>, node: &FpExpr, common_subexprs: &HashMap<u64, Vec<Id>>, hash_cache: &mut HashMap<Id, u64>, bindings: &mut HashMap<u64, String>, let_defs: &mut Vec<(String, String)>) -> (f64, String) {
        let mut cost_fn = UnitCost;
        let mut child_strs = Vec::new();
        let mut total_cost = 0.0;
        for &c in node.children() {
            let (c_cost, c_str) = helper(expr, c, common_subexprs, hash_cache, bindings, let_defs);
            total_cost += c_cost;
            child_strs.push(c_str);
        }
        let this_cost = cost_fn.cost(node, |_| 0.0);
        let s = if child_strs.is_empty() {
            format!("{}", node)
        } else {
            format!("({} {})", node.to_string(), child_strs.join(" "))
        };
        (this_cost + total_cost, s)
    }

    visit_subtree(expr, expr.root(), &mut hash_cache, &mut common_subexprs);
    let (cost, main_expr) = helper(expr, expr.root(), &common_subexprs, &mut hash_cache, &mut bindings, &mut let_defs);

    let mut lets = String::new();
    for (name, rhs) in &let_defs {
        lets.push_str(&format!("let {} = {}\n", name, rhs));
    }
    (cost, format!("{}{}", lets, main_expr))
}

fn main() {
    let benchmarks = load_benchmarks("benchmarks");
    let rules: &[Rewrite<FpExpr, ()>] = &[
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

        // Fp2 Rules
        rw!("mul_const_fp2"; "(*2 2 ?a)" => "(+2 ?a ?a)"),
        rw!("mul_const2_fp2"; "(constmul2 2 ?a)" => "(+2 ?a ?a)"),
        rw!("mulsquare_fp2"; "(*2 ?x ?x)" => "(square2 ?x)"),
        rw!("mul_to_constmul_fp2"; "(*2 ?a ?b)" => "(constmul2 ?a ?b)" if is_const("?a", "?b")),
        rw!("inv_fp2_short";
            "(inv2 (Fp2 ?x ?y))" 
            => 
            "(Fp2 
                (* ?x (inv (+ (square ?x) (square ?y)))) 
                (* (- 0 ?y) (inv (+ (square ?x) (square ?y))))
            )"
        ),
        rw!("commute-add_fp2"; "(+2 ?a ?b)" => "(+2 ?b ?a)"),
        rw!("commute-mul_fp2"; "(*2 ?a ?b)" => "(*2 ?b ?a)"),
        rw!("assoc-add_fp2"; "(+2 (+2 ?a ?b) ?c)" => "(+2 ?a (+2 ?b ?c))"),
        rw!("assoc-mul_fp2"; "(*2 (*2 ?a ?b) ?c)" => "(*2 ?a (*2 ?b ?c))"),
    ];

    for (name, expr) in benchmarks {
        println!("\n=== Benchmark: {} ===", name);

        let runner = Runner::default()
            .with_expr(&expr)
            .run(rules);

        let extractor = Extractor::new(&runner.egraph, UnitCost);
        let (_, best_expr) = extractor.find_best(runner.roots[0]);
        let (best_cost, cse_expr) = extract_common_subexpressions(&best_expr);

        println!("Original expr: {}", expr);
        println!("Optimized expr:\n{}", cse_expr);
        println!("Optimized cost: {}", best_cost);
    }
}