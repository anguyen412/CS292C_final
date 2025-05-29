use egg::{*, rewrite as rw};

define_language! {
    enum Fp4Expr {
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "-" = Sub([Id; 2]),
        "constmul" = ConstMul([Id; 2]),
        "square" = Square(Id),
        Const(u64),
        Symbol(Symbol),
    }
}

struct UnitCost;

impl CostFunction<Fp4Expr> for UnitCost {
    type Cost = usize;

    fn cost<C>(&mut self, enode: &Fp4Expr, mut costs: C) -> usize
    where
        C: FnMut(Id) -> usize,
    {
        let base_cost = match enode {
            Fp4Expr::Add(_) => 1,      // cost for "+"
            Fp4Expr::Mul(_) => 10,      // cost for "*"
            Fp4Expr::ConstMul(_) => 4,
            Fp4Expr::Sub(_) => 1,      // cost for "-"
            Fp4Expr::Square(_) => 6,   // cost for "square"
            Fp4Expr::Symbol(_) => 0,   // no cost for symbols (variables/constants)
            Fp4Expr::Const(_) => 0,
        };

        base_cost + enode.children().iter().map(|&id| costs(id)).sum::<usize>()
    }
}

fn is_const(var: &str) -> impl Fn(&mut EGraph<Fp4Expr, ()>, Id, &Subst) -> bool {
    let var_id = var.parse().unwrap();
    move |egraph, _, subst| {
        // Get the eclass id for the variable
        let eclass_id = subst[var_id];

        // Check if the eclass contains a Const node
        egraph[eclass_id].nodes.iter().any(|node| matches!(node, Fp4Expr::Const(_)))
    }
}

fn main() {
    let rules: &[Rewrite<Fp4Expr, ()>] = &[
        rw!("square_add"; "(square (+ ?a ?b))" => "(+ (+ (square ?a) (square ?b)) (* 2 (* ?a ?b)))"),
        rw!("mul_const"; "(* 2 ?a)" => "(+ ?a ?a)"),
        rw!("mulsquare"; "(* ?x ?x)" => "(square ?x)"),
        rw!(
            "two_xy_to_squares";
            "(* 2 (* ?x ?y))" => "(- (square (+ ?x ?y)) (+ (square ?x) (square ?y)))" // 2xy = (x+y)^2 - x^2 - y^2
        ),
        rw!("mul_to_constmul"; "(* ?a ?b)" => "(constmul ?a ?b)" if is_const("?a")),
    ];

    let expr = "(* 3 x)";
    let expr: RecExpr<Fp4Expr> = expr.parse().unwrap();

    let mut egraph = EGraph::default();
    let id = egraph.add_expr(&expr);

    let runner = Runner::default().with_egraph(egraph).run(rules);
    let cost_fn = UnitCost;

    let extractor = Extractor::new(&runner.egraph, cost_fn);
    let (best_cost, best_expr) = extractor.find_best(id);

    println!("Original expr: {}", expr);
    println!("Optimized expr: {}", best_expr);
    println!("Cost: {}", best_cost);
}