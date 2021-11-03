# Basic Operators

Circom provides boolean, arithmetic, and bitwise operators. They have the standard semantics but the arithmetic operators applied to numeric values work modulo p. 

The precedence and association of the operators are like in Rust (defined [here](https://doc.rust-lang.org/1.22.1/reference/expressions/operator-expr.html#operator-precedence)).

Expressions can be built using the next operators, but the conditional operator `?_:_` can only occur at the top level. 

## Field Elements

A field element is a value in the domain of Z/pZ, where p is the prime number set by default to 

`p = 21888242871839275222246405745257275088548364400416034343698204186575808495617.`

As such, field elements are operated in arithmetic modulo p.

The circom language is parametric to this number, and it can be changed without affecting the rest of the language (using `GLOBAL_FIELD_P`).

## Conditional expressions

**Boolean\_condition ? true\_value : false\_value**

```text
var z = x>y? x : y;
```

This conditional expression is not allowed in a nested form, hence can only be used at the top level.  


## Boolean operators

Next boolean operators are allowed:

| Operator | Example | Explanation |
| :--- | :--- | :--- |
| && | a && b | Boolean operator AND |
| \|\| | a \|\| b | Boolean operator OR |
| ! | ! a | Boolean operator NEGATION |

## Relational operators

The definition of relational operators **`< , > , <= , >= , == , !=`**  depends on the mathematical function ```val(x)``` which is defined as follows:         

           val(z) = z-p  if p/2 +1 <= z < p

           val(z) = z,    otherwise.

According to this function, the definition of the relational operators is as follows:

    `x < y` is defined as val(x % p) < val(y % p)  

    `x > y` is defined as val(x % p) > val(y % p)  

    `x <= y` is defined as val(x % p) <= val(y % p)  

    `x >= y` is defined as val(x % p) >= val(y % p)   

where ```<, >, <=, >=``` are the comparison of integers.



## Arithmetic operators

All arithmetic operations work modulo p. We have the next operators:

| Operator | Example | Explanation |
| :---: | :---: | :---: |
| + | a + b | Arithmetic addition modulo p |
| - | a - b | Arithmetic subtraction modulo p |
| \* | a \* b | Arithmetic multiplication modulo p |
| \*\* | a \*\* b | Power modulo p |
| / | a / b | Multiplication by the inverse modulo p |
| \ | a \ b | Quotient of the integer division |
| % | a % b | Remainder of the integer division |

There are operators that combine arithmetic operators with a final assignment.

| Operator | Example | Explanation |
| :---: | :---: | :---: |
| += | a += b | Arithmetic addition modulo p and assignment |
| -= | a -= b | Arithmetic subtraction modulo p and assignment |
| \*= | a \*= b | Arithmetic multiplication modulo p and assignment |
| \*\*= | a \*\* b | Power modulo p and assignment |
| /= | a /= b | Multiplication by the inverse modulo p and assignment |
| \= | a \= b | Quotient of the integer division and assignment |
| %= | a %= b | Remainder of the integer division and assignment  |
| ++ | a++ | Unit increment. Syntactic sugar for a += 1 |
| -- | a-- | Unit decrement. Syntactic sugar for a -= 1 |

## Bitwise operators

All bitwise operators are performed modulo p.

| Operator | Example | Explanation |
| :--- | :--- | :--- |
| & | a & b | Bitwise AND |
| \| | a \| b | Bitwise OR |
| ~ | ~a | Complement 254 bits |
| ^ | a ^ b | XOR  254 bits |
| &gt;&gt; | a &gt;&gt; 4 | Right shift operator |
| &lt;&lt; | a &lt;&lt; 4 | Left shift operator |

 The shift operations also work modulo p and are defined as follows (assuming p&gt;=7). 

For all ```k``` with ```0=< k <= p/2``` (integer division) we have that 

* ```x >> k = x/(2**k)``` 
*  ```x << k = (x*(2{**}k)~ & ~mask) % p  ``` 

where b is the number of significant bits of p and mask is ```2{**}b - 1```.

For all ```k``` with ```p/2 +1<= k < p``` we have that

* ```x >> k = x << (p-k)``` 
* ```x << k = x >> (p-k)``` 

note that ```k``` is also the negative number ```k-p```.

There are operators that combine bitwise operators with a final assignment.

| Operator | Example | Explanation |
| :--- | :--- | :--- |
| &= | a &= b | Bitwise AND and assignment |
| \|= | a \|= b | Bitwise OR and assignment |
| ~= | ~=a | Complement 254 bits and assignment |
| ^= | a ^= b | XOR  254 bits and assignment |
| &gt;&gt;= | a &gt;&gt;= 4 | Right shift operator and assignment |
| &lt;&lt;= | a &lt;&lt;= 4 | Left shift operator and assignment |

## Examples using operators from the circom library

In the following, there are several examples using combinations of the previous operators.

```text
pragma circom 2.0.0;

template IsZero() {
    signal input in;
    signal output out;
    signal inv;
    inv <-- in!=0 ? 1/in : 0;
    out <== -in*inv +1;
    in*out === 0;
}

component main {public [in]}= IsZero();
```

This template checks if the input signal `in` is `0`. In case it is, the value of output signal`out` is `1`. `0`, otherwise. Note here that we use the intermediate signal `inv` to compute the inverse of the value of `in` or `0` if it does not exist. If `in`is 0, then `in*inv` is 0, and the value of `out` is `1`. Otherwise, `in*inv` is always `1`, then `out` is `0`.

```text
pragma circom 2.0.0;

template Num2Bits(n) {
    signal input in;
    signal output out[n];
    var lc1=0;
    var e2=1;
    for (var i = 0; i<n; i++) {
        out[i] <-- (in >> i) & 1;
        out[i] * (out[i] -1 ) === 0;
        lc1 += out[i] * e2;
        e2 = e2+e2;
    }
    lc1 === in;
}

component main {public [in]}= Num2Bits(3);
```

This templates returns a n-dimensional array with the value of `in` in binary. Line 7 uses the right shift `>>` and operator `&` to obtain at each iteration the `i` component of the array. Finally, line 12 adds the constraint `lc1 = in` to guarantee  that the conversion is well done.

