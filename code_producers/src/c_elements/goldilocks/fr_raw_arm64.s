    .global Fr_rawAdd
    .global Fr_rawAddLS
    .global Fr_rawSub
    .global Fr_rawSubRegular
    .global Fr_rawNeg
    .global Fr_rawNegLS
    .global Fr_rawSubSL
    .global Fr_rawSubLS
    .global Fr_rawMMul
    .global Fr_rawMMul1
    .global Fr_rawFromMontgomery
    .global Fr_rawCopy
    .global Fr_rawSwap
    .global Fr_rawIsEq
    .global Fr_rawIsZero
    .global Fr_rawCopyS2L
    .global Fr_rawCmp
    .global Fr_rawAnd
    .global Fr_rawOr
    .global Fr_rawXor
    .global Fr_rawShr
    .global Fr_rawShl
    .global Fr_rawNot

    .global _Fr_rawAdd
    .global _Fr_rawAddLS
    .global _Fr_rawSub
    .global _Fr_rawSubRegular
    .global _Fr_rawNeg
    .global _Fr_rawNegLS
    .global _Fr_rawSubSL
    .global _Fr_rawSubLS
    .global _Fr_rawMMul
    .global _Fr_rawMMul1
    .global _Fr_rawFromMontgomery
    .global _Fr_rawCopy
    .global _Fr_rawSwap
    .global _Fr_rawIsEq
    .global _Fr_rawIsZero
    .global _Fr_rawCopyS2L
    .global _Fr_rawCmp
    .global _Fr_rawAnd
    .global _Fr_rawOr
    .global _Fr_rawXor
    .global _Fr_rawShr
    .global _Fr_rawShl
    .global _Fr_rawNot

    .text
    .align 4

Fr_rawAdd:
_Fr_rawAdd:
        ldr    x8, [x1]
        ldr    x4, [x2]
        adds   x8,  x8,  x4

        cset   x2,  cs

        adr    x3, Fr_rawq
        ldr    x9, [x3]
        subs   x9,  x8,  x9

        cbnz   x2, Fr_rawAdd_done_s
        b.hs  Fr_rawAdd_done_s

        str    x8, [x0]

        b     Fr_rawAdd_out

Fr_rawAdd_done_s:
        str    x9, [x0]

Fr_rawAdd_out:
        ret


Fr_rawAddLS:
_Fr_rawAddLS:
        ldr    x8, [x1]
        adds   x8,  x8,  x2

        cset   x2,  cs

        adr    x3, Fr_rawq
        ldr    x9, [x3]
        subs   x9,  x8,  x9

        cbnz   x2, Fr_rawAddLS_done_s
        b.hs  Fr_rawAddLS_done_s

        str    x8, [x0]

        b     Fr_rawAddLS_out

Fr_rawAddLS_done_s:
        str    x9, [x0]

Fr_rawAddLS_out:
        ret


Fr_rawSub:
_Fr_rawSub:
        ldr    x8, [x1]
        ldr    x4, [x2]
        subs   x8,  x8,  x4

        b.cs  Fr_rawSub_done

        adr    x3, Fr_rawq
        ldr    x4, [x3]
        adds   x8,  x8,  x4

Fr_rawSub_done:
        str    x8, [x0]
        ret


Fr_rawSubSL:
_Fr_rawSubSL:
        ldr    x8, [x2]
        subs   x8,  x1,  x8

        b.cs  Fr_rawSubSL_done

        adr    x3, Fr_rawq
        ldr    x4, [x3]
        adds   x8,  x8,  x4

Fr_rawSubSL_done:
        str    x8, [x0]
        ret


Fr_rawSubLS:
_Fr_rawSubLS:
        ldr    x8, [x1]
        subs   x8,  x8,  x2

        b.cs  Fr_rawSubLS_done

        adr    x3, Fr_rawq
        ldr    x4, [x3]
        adds   x8,  x8,  x4

Fr_rawSubLS_done:
        str    x8, [x0]
        ret


Fr_rawSubRegular:
_Fr_rawSubRegular:
        ldr    x4, [x1]
        ldr    x8, [x2]
        subs   x4,  x4,  x8
        str    x4, [x0]

        ret


Fr_rawNeg:
_Fr_rawNeg:
        mov    x2, xzr
        ldr    x8, [x1]
        orr    x2,  x2,  x8

        cbz    x2, Fr_rawNeg_done_zero

        adr    x3, Fr_rawq
        ldr    x4, [x3]
        subs   x8,  x4,  x8
        str    x8, [x0]

        ret

Fr_rawNeg_done_zero:
        str   xzr, [x0]

        ret


Fr_rawNegLS:
_Fr_rawNegLS:
        adr    x3, Fr_rawq
        ldr    x8, [x3]
        subs   x9,  x8,  x2

        cset   x2,  cs

        ldr    x4, [x1]
        subs   x9,  x9,  x4

        cset   x3,  cs
        orr    x3,  x3,  x2

        cbz    x3, Fr_rawNegLS_done

        adds   x9,  x9,  x8

Fr_rawNegLS_done:
        str    x9, [x0]
        ret


Fr_rawMMul:
_Fr_rawMMul:
        ldr   x11, [x2]

        adr    x4, Fr_np
        ldr    x4, [x4]

        adr    x6, Fr_rawq
        ldr   x12, [x6]

        // product0 = pRawB * pRawA[0]
        ldr    x3, [x1]
        mul    x9, x11,  x3
        umulh x10, x11,  x3

        // np0 = Fq_np * product0[0]
        mul    x5,  x4,  x9

        // product0 = product0 + Fq_rawq * np0
        mul    x7, x12,  x5
        adds   x9,  x9,  x7
        adcs  x10, x10, xzr
        adc    x8, xzr, xzr

        umulh  x7, x12,  x5
        adds  x10, x10,  x7
        adc    x8,  x8, xzr

        // result ge Fr_rawq
        subs  x11, x10, x12

        cinc   x8,  x8,  hs
        cmp    x8,   1

        csel  x10, x11, x10,  hs
        str   x10, [x0]
        ret


Fr_rawMMul1:
_Fr_rawMMul1:
        ldr   x11, [x1]

        adr    x4, Fr_np
        ldr    x4, [x4]

        adr    x6, Fr_rawq
        ldr   x12, [x6]

        // product0 = pRawB * pRawA
        mul    x9, x11,  x2
        umulh x10, x11,  x2

        // np0 = Fq_np * product0[0]
        mul    x5,  x4,  x9
        // product0 = product0 + Fq_rawq * np0
        mul    x7, x12,  x5
        adds   x9,  x9,  x7
        adc   x10, x10, xzr

        umulh  x7, x12,  x5
        adds  x10, x10,  x7
        adc    x8, xzr, xzr

        // result ge Fr_rawq
        subs  x11, x10, x12

        cinc   x8,  x8,  hs
        cmp    x8,   1

        csel  x10, x11, x10,  hs
        str   x10, [x0]
        ret


Fr_rawFromMontgomery:
_Fr_rawFromMontgomery:
        ldp    x9, x10, [x1]
        mov   x10, xzr

        adr    x4, Fr_np
        ldr    x4, [x4]

        adr    x6, Fr_rawq
        ldr   x12, [x6]

        // np0 = Fq_np * product0[0]
        mul    x5,  x4,  x9
        // product0 = product0 + Fq_rawq * np0
        mul    x7, x12,  x5
        adds   x9,  x9,  x7
        adc   x10, x10, xzr

        umulh  x7, x12,  x5
        adds  x10, x10,  x7
        adc    x8, xzr, xzr

        // result ge Fr_rawq
        subs  x11, x10, x12

        cinc   x8,  x8,  hs
        cmp    x8,   1

        csel  x10, x11, x10,  hs
        str   x10, [x0]
        ret


Fr_rawIsZero:
_Fr_rawIsZero:
        ldr    x1, [x0]

        cmp    x1, xzr
        cset   x0,  eq
        ret

Fr_rawIsEq:
_Fr_rawIsEq:
        ldr    x5, [x0]
        ldr    x9, [x1]
        eor    x2,  x5,  x9

        cmp    x2, xzr
        cset   x0,  eq
        ret

Fr_rawCmp:
_Fr_rawCmp:
        ldr    x3, [x0]
        ldr    x7, [x1]
        subs   x3,  x3,  x7
        cset   x2,  ne

        cneg   x0,  x2,  lo
        ret

Fr_rawCopy:
_Fr_rawCopy:
        ldr    x2, [x1]
        str    x2, [x0]
        ret

Fr_rawCopyS2L:
_Fr_rawCopyS2L:
        cmp    x1, xzr
        b.lt  Fr_rawCopyS2L_adjust_neg

        str    x1, [x0]
        ret

Fr_rawCopyS2L_adjust_neg:
        adr    x3, Fr_rawq

        ldr    x4, [x3]
        adds  x10,  x1,  x4
        str   x10, [x0]

        ret

Fr_rawSwap:
_Fr_rawSwap:
        ldr    x2, [x0]
        ldr   x10, [x1]
        str    x2, [x1]
        str   x10, [x0]
        ret

Fr_rawAnd:
_Fr_rawAnd:
        ldr    x8, [x1]
        ldr    x4, [x2]
        and    x8,  x8,  x4

        adr    x2, Fr_lboMask
        ldr    x2, [x2]
        and    x8,  x8,  x2

        adr    x3, Fr_rawq
        ldr    x9, [x3]
        subs   x9,  x8,  x9

        csel   x8,  x9,  x8,  hs
        str    x8, [x0]
        ret

Fr_rawOr:
_Fr_rawOr:
        ldr    x8, [x1]
        ldr    x4, [x2]
        orr    x8,  x8,  x4

        adr    x2, Fr_lboMask
        ldr    x2, [x2]
        and    x8,  x8,  x2

        adr    x3, Fr_rawq
        ldr    x9, [x3]
        subs   x9,  x8,  x9

        csel   x8,  x9,  x8,  hs
        str    x8, [x0]
        ret

Fr_rawXor:
_Fr_rawXor:
        ldr    x8, [x1]
        ldr    x4, [x2]
        eor    x8,  x8,  x4

        adr    x2, Fr_lboMask
        ldr    x2, [x2]
        and    x8,  x8,  x2

        adr    x3, Fr_rawq
        ldr    x9, [x3]
        subs   x9,  x8,  x9

        csel   x8,  x9,  x8,  hs
        str    x8, [x0]
        ret

Fr_rawNot:
_Fr_rawNot:
        ldr    x8, [x1]
        mvn    x8,  x8

        adr    x2, Fr_lboMask
        ldr    x2, [x2]
        and    x8,  x8,  x2

        adr    x3, Fr_rawq
        ldr    x9, [x3]
        subs   x9,  x8,  x9

        csel   x8,  x9,  x8,  hs
        str    x8, [x0]
        ret

Fr_rawShr:
_Fr_rawShr:
        ldr    x3, [x1]
        lsr    x3,  x3,  x2
        str    x3, [x0]
        ret

Fr_rawShl:
_Fr_rawShl:
        ldr    x3, [x1]
        lsl    x3,  x3,  x2
        str    x3, [x0]
        ret



    .align 8
Fr_rawq:    .quad 0xffffffff00000001
Fr_np:      .quad 0xfffffffeffffffff
Fr_lboMask: .quad 0xffffffffffffffff
