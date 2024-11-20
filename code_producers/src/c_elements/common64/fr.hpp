#define Fr_N64 1
#define Fr_prime 18446744069414584321 // 2**64 - 2**32 + 1
#define Fr_half 9223372034707292160 // 2**64 - 2**32 + 1
// phi = 2**32, phi**2 - phi + 1
// uint32_t ticks32_auto = (uint32_t) ticks64;

inline uint64_t Fr_sum (uint64_t a, uint64_t b) {
  if (a <= Fr_half) {
    if (b <= Fr_half) {
      return a + b;
    } else {
      uint64_t bn = Fr_prime - b;
      if (bn < a) return a - bn;
      else return bn + a;
    }
  } else {
    uint64_t an = Fr_prime - a;
    if (b <= Fr_half) {
      if (an < b) return b - an;
      else return an + b;
    } else return b - an;
      
    }
}

//Assume prime is in (2**64, 2^64 - 2^33 + 1 )
//For instance goldilocks 2^64 - 2^32 + 1
//Multiplying 2 32 bits number is below 2^64 - 2^33 + 1
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
  

}
