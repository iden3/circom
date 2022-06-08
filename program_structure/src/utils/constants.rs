use num_bigint::BigInt;

const P_BN128: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";
const P_BLS12381: &str = 
    "52435875175126190479447740508185965837690552500527637822603658699938581184513";
const P_GOLDILOCKS: &str = 
    "18446744069414584321";
//const P_STR: &str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";

pub struct UsefulConstants {
    p: BigInt,
}

impl Clone for UsefulConstants {
    fn clone(&self) -> Self {
        UsefulConstants { p: self.p.clone() }
    }
}



// impl Default for UsefulConstants {
//     fn default() -> Self {
//         let possible_prime : String = String::from("bn128");
//         let prime_to_use = if possible_prime.eq("bn128") {P_BN128} 
//           else if possible_prime.eq("bls12381") { P_BLS12381} 
//           else {P_GOLDILOCKS};

//         UsefulConstants { p: BigInt::parse_bytes(prime_to_use.as_bytes(), 10).expect("can not parse p") }
//     }
// }

impl UsefulConstants {
    pub fn new(possible_prime: &String) -> UsefulConstants {
        let prime_to_use = if possible_prime.eq("bn128") {P_BN128} 
          else if possible_prime.eq("bls12381") { P_BLS12381} 
          else {P_GOLDILOCKS};

        UsefulConstants { p: BigInt::parse_bytes(prime_to_use.as_bytes(), 10).expect("can not parse p") }
    }
    
    pub fn get_p(&self) -> &BigInt {
        &self.p
    }
}
