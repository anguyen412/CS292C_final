use std::{collections::HashMap, fs, mem::Discriminant};

use egg::{*};
use serde_json::Value;

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

pub struct FpCost {
    cost_map: HashMap<Discriminant<FpExpr>, f64>,
}

impl FpCost {
    pub fn new(cost_json_path: &str) -> Self {
        let file_content = fs::read_to_string(cost_json_path).expect("Failed to read cost json");
        let json: Value = serde_json::from_str(&file_content).expect("Failed to parse cost json");
        let mut cost_map = HashMap::new();

        macro_rules! insert_cost {
            ($field:expr, $op:expr, $variant:path) => {
                if let Some(val) = json[$field][$op].as_f64() {
                    let disc = std::mem::discriminant(&$variant(Default::default()));
                    cost_map.insert(disc, val);
                }
            };
        }

        // Fp
        insert_cost!("Fp", "add", FpExpr::Add);
        insert_cost!("Fp", "add", FpExpr::Sub);
        insert_cost!("Fp", "mult", FpExpr::Mul);
        insert_cost!("Fp", "const_mult", FpExpr::ConstMul);
        insert_cost!("Fp", "square", FpExpr::Square);
        insert_cost!("Fp", "inv", FpExpr::Inv);

        // Fp2
        insert_cost!("Fp2", "add", FpExpr::AddFp2);
        insert_cost!("Fp2", "add", FpExpr::SubFp2);
        insert_cost!("Fp2", "mult", FpExpr::MulFp2);
        insert_cost!("Fp2", "const_mult", FpExpr::ConstMulFp2);
        insert_cost!("Fp2", "square", FpExpr::SquareFp2);
        insert_cost!("Fp2", "inv", FpExpr::InvFp2);

        // Fp4
        insert_cost!("Fp4", "add", FpExpr::AddFp4);
        insert_cost!("Fp4", "add", FpExpr::SubFp4);
        insert_cost!("Fp4", "mult", FpExpr::MulFp4);
        insert_cost!("Fp4", "const_mult", FpExpr::ConstMulFp4);
        insert_cost!("Fp4", "square", FpExpr::SquareFp4);

        // Fp6
        insert_cost!("Fp6", "add", FpExpr::AddFp6);
        insert_cost!("Fp6", "add", FpExpr::SubFp6);
        insert_cost!("Fp6", "mult", FpExpr::MulFp6);
        insert_cost!("Fp6", "const_mult", FpExpr::ConstMulFp6);
        insert_cost!("Fp6", "square", FpExpr::SquareFp6);

        // Fp12
        insert_cost!("Fp12", "add", FpExpr::AddFp12);
        insert_cost!("Fp12", "add", FpExpr::SubFp12);
        insert_cost!("Fp12", "mult", FpExpr::MulFp12);
        insert_cost!("Fp12", "const_mult", FpExpr::ConstMulFp12);
        insert_cost!("Fp12", "square", FpExpr::SquareFp12);

        FpCost { cost_map }
    }
}

impl CostFunction<FpExpr> for FpCost {
    type Cost = f64;

    fn cost<C>(&mut self, enode: &FpExpr, mut costs: C) -> f64
    where
        C: FnMut(Id) -> f64,
    {
        let base_cost = self.cost_map
            .get(&std::mem::discriminant(enode))
            .cloned()
            .unwrap_or(0.0);
        base_cost + enode.children().iter().map(|&id| costs(id)).sum::<f64>()
    }
}