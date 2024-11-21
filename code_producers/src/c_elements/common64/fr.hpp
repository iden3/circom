#include <gmp.h>
#include <stdlib.h>
#include <sstream>

#define Fr_N64 1
#define Fr_prime 18446744069414584321 // 2**64 - 2**32 + 1
#define Fr_half 9223372034707292160 // 2**64 - 2**32 + 1
// phi = 2**32, phi**2 - phi + 1
// uint32_t ticks32_auto = (uint32_t) ticks64;


inline int Fr_toInt(uint64_t a) {
  return (int)a;
}

inline uint64_t Fr_str2element(char const *s, uint base) {
  return strtoul(s, NULL, base);
}

inline char *Fr_element2str(uint64_t a) {
  std::stringstream ss;
  ss << a;
  std::string str = ss.str();
  char * cstr = new char [str.length()+1];
  std::strcpy (cstr, str.c_str());
  return cstr();
}

inline uint64_t Fr_sum (uint64_t a, uint64_t b) {
  if (a <= Fr_half) {
    if (b > Fr_half) {
      uint64_t bn = Fr_prime - b;
      if (bn < a) return a - bn; // is in [0..Fr_prime)
    }
    return a + b; // in the remaining cases a + b < Fr_prime
  } else {
    uint64_t an = Fr_prime - a; // an <= half
    if (an >= b) return a + b;   // b <= half and a + b < Fr_prime
    return b - an; // is in [0..Fr_prime)  
  }
}

inline uint64_t Fr_sub (uint64_t a, uint64_t b) {
  return (b <= a)? a - b : Fr_prime - (b - a); 
}

//Assume prime is in (2**64, 2^64 - 2^33 + 1 )
//For instance goldilocks 2^64 - 2^32 + 1
//Multiplying 2 32 bits number is below 2^64 - 2^33 + 1, hence below prime
inline uint64_t Fr_mul(uint64_t a, uint64_t b) {
  uint64_t a0 = (uint32_t)a;
  uint64_t a1 = a >> 32;
  // a = a1*2^32 + a0
  uint64_t b0 = (uint32_t)b;
  uint64_t b1 = b >> 32;
  // b = b1*2^32 + b0
  uint64_t a0b0 = (a0 * b0); //by assumption below prime
  uint64_t a0b1 = (a0 * b1); //by assumption below prime
  uint64_t a1b0 = (a1 * b0); //by assumption below prime
  uint64_t a1b1 = (a1 * b1); //by assumption below prime
  return Fr_sum(Fr_sum(a1b1,a1b0),Fr_sum(a0b1,a0b0));

}

inline uint64_t Fr_div(uint64_t a, uint64_t b) {
  uint64_t ib = Fr_inv(b);
  return Fr_mul(a,ib);
  
}

inline uint64_t Fr_inv(uint64_t a) {
  uint32_t a0 = (uint32_t)a;
  uint32_t a1 = (uint32_t)(a >> 32);
  mpz_t ma;
  mpz_init_set_ui(ma, a1);
  mpz_mul_2exp(ma, ma, 32);
  mpz_add_ui(ma, a0);
  mpz_t mr;
  mpz_init(mr);
  mpz_invert(mr, ma, Fr_prime);
  a0 = mpz_get_ui(mr);
  mpz_tdiv_q_2exp(mr,mr,32);
  a1 = mpz_get_ui(mr);
  mpz_clear(ma);
  mpz_clear(mr);
  a = (uint64_t)a1;
  a += (uint64_t)a0;
  return a;
}

inline uint64_t Fr_idiv(uint64_t a, uint64_t b) {
  return a / b;
}

inline uint64_t Fr_imod(uint64_t a, uint64_t b) {
  return a % b;
}

inline uint64_t Fr_pow(uint64_t a, uint64_t b) {
  uint64_t p = 1;
  while (b>0) {
    if (b%2 == 0)  {
      a = Fr_mul(a,a);
      b = b / 2;
    } else {
      p = Fr_mul(p,a);
      b = b - 1;
    }
  }
  return p;
}

inline uint64_t Fr_shl(uint64_t a, uint64_t b) {
  if (b > Fr_half) return Fr_shr(a,Fr_prime-b);
  else {
    uint64_t s = a << b;
    if (s >= Fr_prime) s -= Fr_prime;
    return s;
}

inline uint64_t Fr_shr(uint64_t a, uint64_t b) {
  if (b > Fr_half) return Fr_shl(a,Fr_prime-b);
  else return a >> b;
}

inline uint64_t Fr_leq(uint64_t a, uint64_t b) {
  if (a <= Fr_half) {
    if (b <= Fr_half) return a <= b;
    else return 0;
  } else {
    if (b <= Fr_half) return 1;
    else return return a <= b;
  }    
}

inline uint64_t Fr_geq(uint64_t a, uint64_t b) {
  if (a <= Fr_half) {
    if (b <= Fr_half) return a >= b;
    else return 1;
  } else {
    if (b <= Fr_half) return 0;
    else return return a >= b;
  }
}

inline uint64_t Fr_lt(uint64_t a, uint64_t b) {
  if (a <= Fr_half) {
    if (b <= Fr_half) return a < b;
    else return 0;
  } else {
    if (b <= Fr_half) return 1;
    else return return a < b;
  }    
}

inline uint64_t Fr_gt(uint64_t a, uint64_t b) {
  if (a <= Fr_half) {
    if (b <= Fr_half) return a > b;
    else return 1;
  } else {
    if (b <= Fr_half) return 0;
    else return return a > b;
  }
}

inline uint64_t Fr_eq(uint64_t a, uint64_t b) {
  return a == b;
}

inline uint64_t Fr_neq(uint64_t a, uint64_t b) {
  return a != b;
}

inline uint64_t Fr_lor(uint64_t a, uint64_t b) {
  return (a == 0) && (a ==0)? 0 : 1; 
}

inline uint64_t Fr_land(uint64_t a, uint64_t b) {
  return (a == 0) || (a ==0)? 0 : 1; 
}

inline uint64_t Fr_bor(uint64_t a, uint64_t b) {
  uint64_t bor = a | b;
  return bor < Fr_prime ? bor : bor - Fr_prime 
}

inline uint64_t Fr_band(uint64_t a, uint64_t b) {
  return a & b;
}

inline uint64_t Fr_bxor(uint64_t a, uint64_t b) {
  return a^b;
}

inline uint64_t Fr_neg(uint64_t a) {
  return Fr_prime - a;
}

inline uint64_t Fr_lnot(uint64_t a) {
  return a == 0? 1 : 0;
}

inline int Fr_isTrue(uint64_t a) {
  return a == 0? 0 : 1;
}

 inline uint64_t Fr_bnot(uint64_t a) {
  uint64_t bnot = ~a;
  return bnot < Fr_prime ? bnot : bnot - Fr_prime 
}


