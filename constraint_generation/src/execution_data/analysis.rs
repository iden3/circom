use circom_algebra::algebra;
use num_bigint::BigInt;

type AExpr = algebra::ArithmeticExpression<String>;

pub struct Analysis {
    reached: Vec<bool>,
    computed_values: Vec<ValuePOS>,
}
impl Analysis {
    pub fn new(id_max: usize) -> Analysis {
        Analysis { reached: vec![false; id_max], computed_values: vec![ValuePOS::Bottom; id_max] }
    }

    pub fn reached(analysis: &mut Analysis, id: usize) {
        analysis.reached[id] = true;
    }
    pub fn is_reached(analysis: &Analysis, id: usize) -> bool {
        analysis.reached[id]
    }

    pub fn computed(analysis: &mut Analysis, id: usize, value: AExpr) {
        if let AExpr::Number { value } = value {
            let new = ValuePOS::Val(value);
            let old = analysis.computed_values[id].clone();
            analysis.computed_values[id] = ValuePOS::least_upper_bound(&old, &new);
        } else {
            analysis.computed_values[id] = ValuePOS::Top;
        }
    }

    pub fn read_computed(analysis: &Analysis, id: usize) -> Option<BigInt> {
        match &analysis.computed_values[id] {
            ValuePOS::Val(v) => Some(v.clone()),
            _ => None,
        }
    }
}

#[derive(Clone)]
enum ValuePOS {
    Bottom,
    Val(BigInt),
    Top,
}

impl ValuePOS {
    pub fn least_upper_bound(l: &ValuePOS, r: &ValuePOS) -> ValuePOS {
        use ValuePOS::*;
        match (l, r) {
            (v, Bottom) | (Bottom, v) => v.clone(),
            (Val(a), Val(b)) if *a == *b => Val(a.clone()),
            _ => Top,
        }
    }
}
