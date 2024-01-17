use num_bigint::BigInt;

const P_BN128: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";
const P_BLS12381: &str = 
    "52435875175126190479447740508185965837690552500527637822603658699938581184513";
const P_GOLDILOCKS: &str = 
    "18446744069414584321";
const P_GRUMPKIN: &str = "21888242871839275222246405745257275088696311157297823662689037894645226208583";
const P_PALLAS: &str = "28948022309329048855892746252171976963363056481941560715954676764349967630337";
const P_VESTA : &str = "28948022309329048855892746252171976963363056481941647379679742748393362948097";
const P_SECQ256R1 : &str = "115792089210356248762697446949407573530086143415290314195533631308867097853951";
//const P_STR: &str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";

pub struct UsefulConstants {
    p: BigInt,
}

impl Clone for UsefulConstants {
    fn clone(&self) -> Self {
        UsefulConstants { p: self.p.clone() }
    }
}

impl UsefulConstants {
    pub fn new(possible_prime: &String) -> UsefulConstants {
        let prime_to_use = if possible_prime.eq("bn128") {P_BN128} 
          else if possible_prime.eq("bls12381") { P_BLS12381} 
          else if possible_prime.eq("goldilocks") { P_GOLDILOCKS} 
          else if possible_prime.eq("grumpkin") { P_GRUMPKIN} 
          else if possible_prime.eq("pallas") { P_PALLAS} 
          else if possible_prime.eq("vesta") { P_VESTA} 
          else if possible_prime.eq("secq256r1") { P_SECQ256R1}
          else {unreachable!()};

        UsefulConstants { p: BigInt::parse_bytes(prime_to_use.as_bytes(), 10).expect("can not parse p") }
    }
    
    pub fn get_p(&self) -> &BigInt {
        &self.p
    }
}
