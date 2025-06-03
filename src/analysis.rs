use egg::{*};
use std::collections::{HashMap};
use std::hash::{Hash, Hasher};

use crate::dsl::{*};

#[derive(Default)]
pub struct ConstantFolding;
impl Analysis<FpExpr> for ConstantFolding {
    type Data = Option<u64>;

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_max(to, from)
    }

    fn make(egraph: &mut EGraph<FpExpr, Self>, enode: &FpExpr) -> Self::Data {
        let x = |i: &Id| egraph[*i].data;
        match enode {
            FpExpr::Const(n) => Some(*n),
            FpExpr::Add([a, b]) => Some(x(a)? + x(b)?),
            FpExpr::Sub([a, b]) => Some(x(a)? - x(b)?),
            FpExpr::ConstMul([a, b]) => Some(x(a)? * x(b)?),
            FpExpr::Square(n) => Some(x(n)? * x(n)?),
            _ => None,
        }
    }

    fn modify(egraph: &mut EGraph<FpExpr, Self>, id: Id) {
        if let Some(i) = egraph[id].data {
            let added = egraph.add(FpExpr::Const(i));
            egraph.union(id, added);
        }
    }
}

pub fn is_const(a: &str, b: &str) -> impl Fn(&mut EGraph<FpExpr, ConstantFolding>, Id, &Subst) -> bool {
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

pub fn extract_common_subexpressions<C: CostFunction<FpExpr, Cost = f64>>(expr: &RecExpr<FpExpr>, mut cf: C) -> (f64, String) {
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

    fn helper<C: CostFunction<FpExpr, Cost = f64>>(
        expr: &RecExpr<FpExpr>,
        id: Id,
        common_subexprs: &HashMap<u64, Vec<Id>>,
        hash_cache: &mut HashMap<Id, u64>,
        bindings: &mut HashMap<u64, String>,
        let_defs: &mut Vec<(String, String)>,
        cf: &mut C,
    ) -> (f64, String) {
        let h = hash_subtree(expr, id, hash_cache);
        let node = &expr[id];

        if let Some(ids) = common_subexprs.get(&h) {
            if ids.len() > 1 && !matches!(node, FpExpr::Const(_) | FpExpr::Symbol(_) | FpExpr::Xi) {
                if let Some(name) = bindings.get(&h) {
                    return (0.0, name.clone());
                } else {
                    let (cost, s) = helper_children(expr, node, common_subexprs, hash_cache, bindings, let_defs, cf);
                    let name = format!("t{}", let_defs.len());
                    bindings.insert(h, name.clone());
                    let_defs.push((name.clone(), s.clone()));
                    return (cost, name);
                }
            }
        }
        helper_children(expr, node, common_subexprs, hash_cache, bindings, let_defs, cf)
    }

    fn helper_children<C: CostFunction<FpExpr, Cost = f64>>(
        expr: &RecExpr<FpExpr>,
        node: &FpExpr,
        common_subexprs: &HashMap<u64, Vec<Id>>,
        hash_cache: &mut HashMap<Id, u64>,
        bindings: &mut HashMap<u64, String>,
        let_defs: &mut Vec<(String, String)>,
        cf: &mut C,
    ) -> (f64, String) {
        let mut child_strs = Vec::new();
        let mut total_cost = 0.0;
        for &c in node.children() {
            let (c_cost, c_str) = helper(expr, c, common_subexprs, hash_cache, bindings, let_defs, cf);
            total_cost += c_cost;
            child_strs.push(c_str);
        }
        let this_cost = cf.cost(node, |_| 0.0);
        let s = if child_strs.is_empty() {
            format!("{}", node)
        } else {
            format!("({} {})", node.to_string(), child_strs.join(" "))
        };
        (this_cost + total_cost, s)
    }

    visit_subtree(expr, expr.root(), &mut hash_cache, &mut common_subexprs);
    let (cost, main_expr) = helper(expr, expr.root(), &common_subexprs, &mut hash_cache, &mut bindings, &mut let_defs, &mut cf);

    let mut lets = String::new();
    for (name, rhs) in &let_defs {
        lets.push_str(&format!("let {} = {}\n", name, rhs));
    }
    (cost, format!("{}{}", lets, main_expr))
}