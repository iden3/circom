use num_bigint::BigInt;

const P_STR: &str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";

pub struct UsefulConstants {
    p: BigInt,
}

impl Clone for UsefulConstants {
    fn clone(&self) -> Self {
        UsefulConstants { p: self.p.clone() }
    }
}
impl Default for UsefulConstants {
    fn default() -> Self {
        UsefulConstants { p: BigInt::parse_bytes(P_STR.as_bytes(), 10).expect("can not parse p") }
    }
}

impl UsefulConstants {
    pub fn new() -> UsefulConstants {
        UsefulConstants::default()
    }
    pub fn get_p(&self) -> &BigInt {
        &self.p
    }
}
