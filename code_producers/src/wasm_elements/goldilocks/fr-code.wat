(func $Fr_int_copy (type $_sig_i32i32)
     (param $px i32)
     (param $pr i32)
    get_local $pr
    get_local $px
    i64.load
    i64.store
)
(func $Fr_int_zero (type $_sig_i32)
     (param $pr i32)
    get_local $pr
    i64.const 0
    i64.store
)
(func $Fr_int_isZero (type $_sig_i32ri32)
     (param $px i32)
    (result i32)
    get_local $px
    i64.load
    i64.eqz
    return
    i32.const 0
    return
)
(func $Fr_int_one (type $_sig_i32)
     (param $pr i32)
    get_local $pr
    i64.const 1
    i64.store
)
(func $Fr_int_eq (type $_sig_i32i32ri32)
     (param $px i32)
     (param $py i32)
    (result i32)
    get_local $px
    i64.load
    get_local $py
    i64.load
    i64.eq
    return
    i32.const 0
    return
)
(func $Fr_int_gt (type $_sig_i32i32ri32)
     (param $px i32)
     (param $py i32)
    (result i32)
    get_local $px
    i64.load
    get_local $py
    i64.load
    i64.gt_u
    return
    i32.const 0
    return
)
(func $Fr_int_gte (type $_sig_i32i32ri32)
     (param $px i32)
     (param $py i32)
    (result i32)
    get_local $px
    i64.load
    get_local $py
    i64.load
    i64.ge_u
    return
    i32.const 0
    return
)
(func $Fr_int_add (type $_sig_i32i32i32ri32)
     (param $x i32)
     (param $y i32)
     (param $r i32)
    (result i32)
     (local $c i64)
    get_local $x
    i64.load32_u
    get_local $y
    i64.load32_u
    i64.add
    set_local $c
    get_local $r
    get_local $c
    i64.store32
    get_local $x
    i64.load32_u offset=4
    get_local $y
    i64.load32_u offset=4
    i64.add
    get_local $c
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c
    get_local $r
    get_local $c
    i64.store32 offset=4
    get_local $c
    i64.const 32
    i64.shr_u
    i32.wrap/i64
)
(func $Fr_int_sub (type $_sig_i32i32i32ri32)
     (param $x i32)
     (param $y i32)
     (param $r i32)
    (result i32)
     (local $c i64)
    get_local $x
    i64.load32_u
    get_local $y
    i64.load32_u
    i64.sub
    set_local $c
    get_local $r
    get_local $c
    i64.const 0xFFFFFFFF
    i64.and
    i64.store32
    get_local $x
    i64.load32_u offset=4
    get_local $y
    i64.load32_u offset=4
    i64.sub
    get_local $c
    i64.const 32
    i64.shr_s
    i64.add
    set_local $c
    get_local $r
    get_local $c
    i64.const 0xFFFFFFFF
    i64.and
    i64.store32 offset=4
    get_local $c
    i64.const 32
    i64.shr_s
    i32.wrap/i64
)
(func $Fr_int_mul (type $_sig_i32i32i32)
     (param $x i32)
     (param $y i32)
     (param $r i32)
     (local $c0 i64)
     (local $c1 i64)
     (local $x0 i64)
     (local $y0 i64)
     (local $x1 i64)
     (local $y1 i64)
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x
    i64.load32_u
    tee_local $x0
    get_local $y
    i64.load32_u
    tee_local $y0
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $r
    get_local $c0
    i64.store32
    get_local $c1
    i64.const 32
    i64.shr_u
    set_local $c0
    get_local $c1
    i64.const 4294967295
    i64.and
    get_local $x0
    get_local $y
    i64.load32_u offset=4
    tee_local $y1
    i64.mul
    i64.add
    set_local $c1
    get_local $c0
    get_local $c1
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c0
    get_local $c1
    i64.const 4294967295
    i64.and
    get_local $x
    i64.load32_u offset=4
    tee_local $x1
    get_local $y0
    i64.mul
    i64.add
    set_local $c1
    get_local $c0
    get_local $c1
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c0
    get_local $r
    get_local $c1
    i64.store32 offset=4
    get_local $c0
    i64.const 32
    i64.shr_u
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x1
    get_local $y1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $r
    get_local $c0
    i64.store32 offset=8
    get_local $c1
    i64.const 32
    i64.shr_u
    set_local $c0
    get_local $r
    get_local $c1
    i64.store32 offset=12
)
(func $Fr_int_square (type $_sig_i32i32)
     (param $x i32)
     (param $r i32)
     (local $c0 i64)
     (local $c1 i64)
     (local $c0_old i64)
     (local $c1_old i64)
     (local $x0 i64)
     (local $x1 i64)
    i64.const 0
    set_local $c0
    i64.const 0
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    i64.const 1
    i64.shl
    set_local $c0
    get_local $c1
    i64.const 1
    i64.shl
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x
    i64.load32_u
    tee_local $x0
    get_local $x0
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $r
    get_local $c0
    i64.store32
    get_local $c1
    set_local $c0_old
    get_local $c0_old
    i64.const 32
    i64.shr_u
    set_local $c1_old
    i64.const 0
    set_local $c0
    i64.const 0
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x0
    get_local $x
    i64.load32_u offset=4
    tee_local $x1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    i64.const 1
    i64.shl
    set_local $c0
    get_local $c1
    i64.const 1
    i64.shl
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $c0_old
    i64.const 4294967295
    i64.and
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    get_local $c1_old
    i64.add
    set_local $c1
    get_local $r
    get_local $c0
    i64.store32 offset=4
    get_local $c1
    set_local $c0_old
    get_local $c0_old
    i64.const 32
    i64.shr_u
    set_local $c1_old
    i64.const 0
    set_local $c0
    i64.const 0
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    i64.const 1
    i64.shl
    set_local $c0
    get_local $c1
    i64.const 1
    i64.shl
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x1
    get_local $x1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $c0_old
    i64.const 4294967295
    i64.and
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    get_local $c1_old
    i64.add
    set_local $c1
    get_local $r
    get_local $c0
    i64.store32 offset=8
    get_local $c1
    set_local $c0_old
    get_local $c0_old
    i64.const 32
    i64.shr_u
    set_local $c1_old
    get_local $r
    get_local $c0_old
    i64.store32 offset=12
)
(func $Fr_int_squareOld (type $_sig_i32i32)
     (param $x i32)
     (param $r i32)
    get_local $x
    get_local $x
    get_local $r
    call $Fr_int_mul
)
(func $Fr_int__mul1 (type $_sig_i32i64i32)
     (param $px i32)
     (param $y i64)
     (param $pr i32)
     (local $c i64)
    get_local $px
    i64.load32_u align=1
    get_local $y
    i64.mul
    set_local $c
    get_local $pr
    get_local $c
    i64.store32 align=1
    get_local $px
    i64.load32_u offset=4 align=1
    get_local $y
    i64.mul
    get_local $c
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c
    get_local $pr
    get_local $c
    i64.store32 offset=4 align=1
)
(func $Fr_int__add1 (type $_sig_i32i64)
     (param $x i32)
     (param $y i64)
     (local $c i64)
     (local $px i32)
    get_local $x
    set_local $px
    get_local $px
    i64.load32_u align=1
    get_local $y
    i64.add
    set_local $c
    get_local $px
    get_local $c
    i64.store32 align=1
    get_local $c
    i64.const 32
    i64.shr_u
    set_local $c
    block
        loop
            get_local $c
            i64.eqz
            br_if 1
            get_local $px
            i32.const 4
            i32.add
            set_local $px
            get_local $px
            i64.load32_u align=1
            get_local $c
            i64.add
            set_local $c
            get_local $px
            get_local $c
            i64.store32 align=1
            get_local $c
            i64.const 32
            i64.shr_u
            set_local $c
            br 0
        end
    end
)
(func $Fr_int_div (type $_sig_i32i32i32i32)
     (param $x i32)
     (param $y i32)
     (param $c i32)
     (param $r i32)
     (local $rr i32)
     (local $cc i32)
     (local $eX i32)
     (local $eY i32)
     (local $sy i64)
     (local $sx i64)
     (local $ec i32)
    get_local $c
    if
        get_local $c
        set_local $cc
    else
        i32.const 72
        set_local $cc
    end
    get_local $r
    if
        get_local $r
        set_local $rr
    else
        i32.const 80
        set_local $rr
    end
    get_local $x
    get_local $rr
    call $Fr_int_copy
    get_local $y
    i32.const 64
    call $Fr_int_copy
    get_local $cc
    call $Fr_int_zero
    i32.const 88
    call $Fr_int_zero
    i32.const 7
    set_local $eX
    i32.const 7
    set_local $eY
    block
        loop
            i32.const 64
            get_local $eY
            i32.add
            i32.load8_u
            get_local $eY
            i32.const 3
            i32.eq
            i32.or
            br_if 1
            get_local $eY
            i32.const 1
            i32.sub
            set_local $eY
            br 0
        end
    end
    i32.const 64
    get_local $eY
    i32.add
    i32.const 3
    i32.sub
    i64.load32_u align=1
    i64.const 1
    i64.add
    set_local $sy
    get_local $sy
    i64.const 1
    i64.eq
    if
        i64.const 0
        i64.const 0
        i64.div_u
        drop
    end
    block
        loop
            block
                loop
                    get_local $rr
                    get_local $eX
                    i32.add
                    i32.load8_u
                    get_local $eX
                    i32.const 7
                    i32.eq
                    i32.or
                    br_if 1
                    get_local $eX
                    i32.const 1
                    i32.sub
                    set_local $eX
                    br 0
                end
            end
            get_local $rr
            get_local $eX
            i32.add
            i32.const 7
            i32.sub
            i64.load align=1
            set_local $sx
            get_local $sx
            get_local $sy
            i64.div_u
            set_local $sx
            get_local $eX
            get_local $eY
            i32.sub
            i32.const 4
            i32.sub
            set_local $ec
            block
                loop
                    get_local $sx
                    i64.const 0xFFFFFFFF00000000
                    i64.and
                    i64.eqz
                    get_local $ec
                    i32.const 0
                    i32.ge_s
                    i32.and
                    br_if 1
                    get_local $sx
                    i64.const 8
                    i64.shr_u
                    set_local $sx
                    get_local $ec
                    i32.const 1
                    i32.add
                    set_local $ec
                    br 0
                end
            end
            get_local $sx
            i64.eqz
            if
                get_local $rr
                i32.const 64
                call $Fr_int_gte
                i32.eqz
                br_if 2
                i64.const 1
                set_local $sx
                i32.const 0
                set_local $ec
            end
            i32.const 64
            get_local $sx
            i32.const 96
            call $Fr_int__mul1
            get_local $rr
            i32.const 96
            get_local $ec
            i32.sub
            get_local $rr
            call $Fr_int_sub
            drop
            get_local $cc
            get_local $ec
            i32.add
            get_local $sx
            call $Fr_int__add1
            br 0
        end
    end
)
(func $Fr_int_inverseMod (type $_sig_i32i32i32)
     (param $px i32)
     (param $pm i32)
     (param $pr i32)
     (local $t i32)
     (local $newt i32)
     (local $r i32)
     (local $qq i32)
     (local $qr i32)
     (local $newr i32)
     (local $swp i32)
     (local $x i32)
     (local $signt i32)
     (local $signnewt i32)
     (local $signx i32)
    i32.const 104
    set_local $t
    i32.const 104
    call $Fr_int_zero
    i32.const 0
    set_local $signt
    i32.const 112
    set_local $r
    get_local $pm
    i32.const 112
    call $Fr_int_copy
    i32.const 120
    set_local $newt
    i32.const 120
    call $Fr_int_one
    i32.const 0
    set_local $signnewt
    i32.const 128
    set_local $newr
    get_local $px
    i32.const 128
    call $Fr_int_copy
    i32.const 136
    set_local $qq
    i32.const 144
    set_local $qr
    i32.const 168
    set_local $x
    block
        loop
            get_local $newr
            call $Fr_int_isZero
            br_if 1
            get_local $r
            get_local $newr
            get_local $qq
            get_local $qr
            call $Fr_int_div
            get_local $qq
            get_local $newt
            i32.const 152
            call $Fr_int_mul
            get_local $signt
            if
                get_local $signnewt
                if
                    i32.const 152
                    get_local $t
                    call $Fr_int_gte
                    if
                        i32.const 152
                        get_local $t
                        get_local $x
                        call $Fr_int_sub
                        drop
                        i32.const 0
                        set_local $signx
                    else
                        get_local $t
                        i32.const 152
                        get_local $x
                        call $Fr_int_sub
                        drop
                        i32.const 1
                        set_local $signx
                    end
                else
                    i32.const 152
                    get_local $t
                    get_local $x
                    call $Fr_int_add
                    drop
                    i32.const 1
                    set_local $signx
                end
            else
                get_local $signnewt
                if
                    i32.const 152
                    get_local $t
                    get_local $x
                    call $Fr_int_add
                    drop
                    i32.const 0
                    set_local $signx
                else
                    get_local $t
                    i32.const 152
                    call $Fr_int_gte
                    if
                        get_local $t
                        i32.const 152
                        get_local $x
                        call $Fr_int_sub
                        drop
                        i32.const 0
                        set_local $signx
                    else
                        i32.const 152
                        get_local $t
                        get_local $x
                        call $Fr_int_sub
                        drop
                        i32.const 1
                        set_local $signx
                    end
                end
            end
            get_local $t
            set_local $swp
            get_local $newt
            set_local $t
            get_local $x
            set_local $newt
            get_local $swp
            set_local $x
            get_local $signnewt
            set_local $signt
            get_local $signx
            set_local $signnewt
            get_local $r
            set_local $swp
            get_local $newr
            set_local $r
            get_local $qr
            set_local $newr
            get_local $swp
            set_local $qr
            br 0
        end
    end
    get_local $signt
    if
        get_local $pm
        get_local $t
        get_local $pr
        call $Fr_int_sub
        drop
    else
        get_local $t
        get_local $pr
        call $Fr_int_copy
    end
)
(func $Fr_F1m_add (type $_sig_i32i32i32)
     (param $x i32)
     (param $y i32)
     (param $r i32)
    get_local $x
    get_local $y
    get_local $r
    call $Fr_int_add
    if
        get_local $r
        i32.const 176
        get_local $r
        call $Fr_int_sub
        drop
    else
        get_local $r
        i32.const 176
        call $Fr_int_gte
        if
            get_local $r
            i32.const 176
            get_local $r
            call $Fr_int_sub
            drop
        end
    end
)
(func $Fr_F1m_sub (type $_sig_i32i32i32)
     (param $x i32)
     (param $y i32)
     (param $r i32)
    get_local $x
    get_local $y
    get_local $r
    call $Fr_int_sub
    if
        get_local $r
        i32.const 176
        get_local $r
        call $Fr_int_add
        drop
    end
)
(func $Fr_F1m_neg (type $_sig_i32i32)
     (param $x i32)
     (param $r i32)
    i32.const 216
    get_local $x
    get_local $r
    call $Fr_F1m_sub
)
(func $Fr_F1m_mReduct (type $_sig_i32i32)
     (param $t i32)
     (param $r i32)
     (local $np32 i64)
     (local $c i64)
     (local $m i64)
    i64.const 4294967295
    set_local $np32
    i64.const 0
    set_local $c
    get_local $t
    i64.load32_u
    get_local $np32
    i64.mul
    i64.const 0xFFFFFFFF
    i64.and
    set_local $m
    get_local $t
    i64.load32_u
    get_local $c
    i64.const 32
    i64.shr_u
    i64.add
    i32.const 176
    i64.load32_u
    get_local $m
    i64.mul
    i64.add
    set_local $c
    get_local $t
    get_local $c
    i64.store32
    get_local $t
    i64.load32_u offset=4
    get_local $c
    i64.const 32
    i64.shr_u
    i64.add
    i32.const 176
    i64.load32_u offset=4
    get_local $m
    i64.mul
    i64.add
    set_local $c
    get_local $t
    get_local $c
    i64.store32 offset=4
    i32.const 272
    get_local $c
    i64.const 32
    i64.shr_u
    i64.store32
    i64.const 0
    set_local $c
    get_local $t
    i64.load32_u offset=4
    get_local $np32
    i64.mul
    i64.const 0xFFFFFFFF
    i64.and
    set_local $m
    get_local $t
    i64.load32_u offset=4
    get_local $c
    i64.const 32
    i64.shr_u
    i64.add
    i32.const 176
    i64.load32_u
    get_local $m
    i64.mul
    i64.add
    set_local $c
    get_local $t
    get_local $c
    i64.store32 offset=4
    get_local $t
    i64.load32_u offset=8
    get_local $c
    i64.const 32
    i64.shr_u
    i64.add
    i32.const 176
    i64.load32_u offset=4
    get_local $m
    i64.mul
    i64.add
    set_local $c
    get_local $t
    get_local $c
    i64.store32 offset=8
    i32.const 272
    get_local $c
    i64.const 32
    i64.shr_u
    i64.store32 offset=4
    i32.const 272
    get_local $t
    i32.const 8
    i32.add
    get_local $r
    call $Fr_F1m_add
)
(func $Fr_F1m_mul (type $_sig_i32i32i32)
     (param $x i32)
     (param $y i32)
     (param $r i32)
     (local $c0 i64)
     (local $c1 i64)
     (local $np32 i64)
     (local $x0 i64)
     (local $y0 i64)
     (local $m0 i64)
     (local $q0 i64)
     (local $x1 i64)
     (local $y1 i64)
     (local $m1 i64)
     (local $q1 i64)
    i64.const 4294967295
    set_local $np32
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x
    i64.load32_u
    tee_local $x0
    get_local $y
    i64.load32_u
    tee_local $y0
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $np32
    i64.mul
    i64.const 0xFFFFFFFF
    i64.and
    set_local $m0
    get_local $c0
    i64.const 4294967295
    i64.and
    i32.const 0
    i64.load32_u offset=176
    tee_local $q0
    get_local $m0
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c1
    i64.const 32
    i64.shr_u
    set_local $c0
    get_local $c1
    i64.const 4294967295
    i64.and
    get_local $x0
    get_local $y
    i64.load32_u offset=4
    tee_local $y1
    i64.mul
    i64.add
    set_local $c1
    get_local $c0
    get_local $c1
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c0
    get_local $c1
    i64.const 4294967295
    i64.and
    get_local $x
    i64.load32_u offset=4
    tee_local $x1
    get_local $y0
    i64.mul
    i64.add
    set_local $c1
    get_local $c0
    get_local $c1
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c0
    get_local $c1
    i64.const 4294967295
    i64.and
    i32.const 0
    i64.load32_u offset=180
    tee_local $q1
    get_local $m0
    i64.mul
    i64.add
    set_local $c1
    get_local $c0
    get_local $c1
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c0
    get_local $c1
    i64.const 4294967295
    i64.and
    get_local $np32
    i64.mul
    i64.const 0xFFFFFFFF
    i64.and
    set_local $m1
    get_local $c1
    i64.const 4294967295
    i64.and
    get_local $q0
    get_local $m1
    i64.mul
    i64.add
    set_local $c1
    get_local $c0
    get_local $c1
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c0
    get_local $c0
    i64.const 32
    i64.shr_u
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x1
    get_local $y1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $q1
    get_local $m1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $r
    get_local $c0
    i64.store32
    get_local $c1
    i64.const 32
    i64.shr_u
    set_local $c0
    get_local $r
    get_local $c1
    i64.store32 offset=4
    get_local $c0
    i32.wrap/i64
    if
        get_local $r
        i32.const 176
        get_local $r
        call $Fr_int_sub
        drop
    else
        get_local $r
        i32.const 176
        call $Fr_int_gte
        if
            get_local $r
            i32.const 176
            get_local $r
            call $Fr_int_sub
            drop
        end
    end
)
(func $Fr_F1m_square (type $_sig_i32i32)
     (param $x i32)
     (param $r i32)
     (local $c0 i64)
     (local $c1 i64)
     (local $c0_old i64)
     (local $c1_old i64)
     (local $np32 i64)
     (local $x0 i64)
     (local $m0 i64)
     (local $q0 i64)
     (local $x1 i64)
     (local $m1 i64)
     (local $q1 i64)
    i64.const 4294967295
    set_local $np32
    i64.const 0
    set_local $c0
    i64.const 0
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    i64.const 1
    i64.shl
    set_local $c0
    get_local $c1
    i64.const 1
    i64.shl
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x
    i64.load32_u
    tee_local $x0
    get_local $x0
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $np32
    i64.mul
    i64.const 0xFFFFFFFF
    i64.and
    set_local $m0
    get_local $c0
    i64.const 4294967295
    i64.and
    i32.const 0
    i64.load32_u offset=176
    tee_local $q0
    get_local $m0
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c1
    set_local $c0_old
    get_local $c0_old
    i64.const 32
    i64.shr_u
    set_local $c1_old
    i64.const 0
    set_local $c0
    i64.const 0
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x0
    get_local $x
    i64.load32_u offset=4
    tee_local $x1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    i64.const 1
    i64.shl
    set_local $c0
    get_local $c1
    i64.const 1
    i64.shl
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $c0_old
    i64.const 4294967295
    i64.and
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    get_local $c1_old
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    i32.const 0
    i64.load32_u offset=180
    tee_local $q1
    get_local $m0
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $np32
    i64.mul
    i64.const 0xFFFFFFFF
    i64.and
    set_local $m1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $q0
    get_local $m1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c1
    set_local $c0_old
    get_local $c0_old
    i64.const 32
    i64.shr_u
    set_local $c1_old
    i64.const 0
    set_local $c0
    i64.const 0
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    i64.const 1
    i64.shl
    set_local $c0
    get_local $c1
    i64.const 1
    i64.shl
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $x1
    get_local $x1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $c0_old
    i64.const 4294967295
    i64.and
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    get_local $c1_old
    i64.add
    set_local $c1
    get_local $c0
    i64.const 4294967295
    i64.and
    get_local $q1
    get_local $m1
    i64.mul
    i64.add
    set_local $c0
    get_local $c1
    get_local $c0
    i64.const 32
    i64.shr_u
    i64.add
    set_local $c1
    get_local $r
    get_local $c0
    i64.store32
    get_local $c1
    set_local $c0_old
    get_local $c0_old
    i64.const 32
    i64.shr_u
    set_local $c1_old
    get_local $r
    get_local $c0_old
    i64.store32 offset=4
    get_local $c1_old
    i32.wrap/i64
    if
        get_local $r
        i32.const 176
        get_local $r
        call $Fr_int_sub
        drop
    else
        get_local $r
        i32.const 176
        call $Fr_int_gte
        if
            get_local $r
            i32.const 176
            get_local $r
            call $Fr_int_sub
            drop
        end
    end
)
(func $Fr_F1m_squareOld (type $_sig_i32i32)
     (param $x i32)
     (param $r i32)
    get_local $x
    get_local $x
    get_local $r
    call $Fr_F1m_mul
)
(func $Fr_F1m_toMontgomery (type $_sig_i32i32)
     (param $x i32)
     (param $r i32)
    get_local $x
    i32.const 192
    get_local $r
    call $Fr_F1m_mul
)
(func $Fr_F1m_fromMontgomery (type $_sig_i32i32)
     (param $x i32)
     (param $r i32)
    get_local $x
    i32.const 304
    call $Fr_int_copy
    i32.const 312
    call $Fr_int_zero
    i32.const 304
    get_local $r
    call $Fr_F1m_mReduct
)
(func $Fr_F1m_isNegative (type $_sig_i32ri32)
     (param $x i32)
    (result i32)
    get_local $x
    i32.const 320
    call $Fr_F1m_fromMontgomery
    i32.const 320
    i32.load
    i32.const 1
    i32.and
)
(func $Fr_F1m_inverse (type $_sig_i32i32)
     (param $x i32)
     (param $r i32)
    get_local $x
    get_local $r
    call $Fr_F1m_fromMontgomery
    get_local $r
    i32.const 176
    get_local $r
    call $Fr_int_inverseMod
    get_local $r
    get_local $r
    call $Fr_F1m_toMontgomery
)
(func $Fr_F1m_one (type $_sig_i32)
     (param $pr i32)
    i32.const 208
    get_local $pr
    call $Fr_int_copy
)
(func $Fr_F1m_load (type $_sig_i32i32i32)
     (param $scalar i32)
     (param $scalarLen i32)
     (param $r i32)
     (local $p i32)
     (local $l i32)
     (local $i i32)
     (local $j i32)
    get_local $r
    call $Fr_int_zero
    i32.const 8
    set_local $i
    get_local $scalar
    set_local $p
    block
        loop
            get_local $i
            get_local $scalarLen
            i32.gt_u
            br_if 1
            get_local $i
            i32.const 8
            i32.eq
            if
                i32.const 328
                call $Fr_F1m_one
            else
                i32.const 328
                i32.const 192
                i32.const 328
                call $Fr_F1m_mul
            end
            get_local $p
            i32.const 328
            i32.const 336
            call $Fr_F1m_mul
            get_local $r
            i32.const 336
            get_local $r
            call $Fr_F1m_add
            get_local $p
            i32.const 8
            i32.add
            set_local $p
            get_local $i
            i32.const 8
            i32.add
            set_local $i
            br 0
        end
    end
    get_local $scalarLen
    i32.const 8
    i32.rem_u
    set_local $l
    get_local $l
    i32.eqz
    if
        return
    end
    i32.const 336
    call $Fr_int_zero
    i32.const 0
    set_local $j
    block
        loop
            get_local $j
            get_local $l
            i32.eq
            br_if 1
            get_local $j
            get_local $p
            i32.load8_u
            i32.store8 offset=336
            get_local $p
            i32.const 1
            i32.add
            set_local $p
            get_local $j
            i32.const 1
            i32.add
            set_local $j
            br 0
        end
    end
    get_local $i
    i32.const 8
    i32.eq
    if
        i32.const 328
        call $Fr_F1m_one
    else
        i32.const 328
        i32.const 192
        i32.const 328
        call $Fr_F1m_mul
    end
    i32.const 336
    i32.const 328
    i32.const 336
    call $Fr_F1m_mul
    get_local $r
    i32.const 336
    get_local $r
    call $Fr_F1m_add
)
(func $Fr_F1m_timesScalar (type $_sig_i32i32i32i32)
     (param $x i32)
     (param $scalar i32)
     (param $scalarLen i32)
     (param $r i32)
    get_local $scalar
    get_local $scalarLen
    i32.const 344
    call $Fr_F1m_load
    i32.const 344
    i32.const 344
    call $Fr_F1m_toMontgomery
    get_local $x
    i32.const 344
    get_local $r
    call $Fr_F1m_mul
)
(func $Fr_F1m_exp (type $_sig_i32i32i32i32)
     (param $base i32)
     (param $scalar i32)
     (param $scalarLength i32)
     (param $r i32)
     (local $i i32)
     (local $b i32)
    get_local $base
    i32.const 352
    call $Fr_int_copy
    get_local $r
    call $Fr_F1m_one
    get_local $scalarLength
    set_local $i
    block
        loop
            get_local $i
            i32.const 1
            i32.sub
            set_local $i
            get_local $scalar
            get_local $i
            i32.add
            i32.load8_u
            set_local $b
            get_local $r
            get_local $r
            call $Fr_F1m_square
            get_local $b
            i32.const 128
            i32.ge_u
            if
                get_local $b
                i32.const 128
                i32.sub
                set_local $b
                i32.const 352
                get_local $r
                get_local $r
                call $Fr_F1m_mul
            end
            get_local $r
            get_local $r
            call $Fr_F1m_square
            get_local $b
            i32.const 64
            i32.ge_u
            if
                get_local $b
                i32.const 64
                i32.sub
                set_local $b
                i32.const 352
                get_local $r
                get_local $r
                call $Fr_F1m_mul
            end
            get_local $r
            get_local $r
            call $Fr_F1m_square
            get_local $b
            i32.const 32
            i32.ge_u
            if
                get_local $b
                i32.const 32
                i32.sub
                set_local $b
                i32.const 352
                get_local $r
                get_local $r
                call $Fr_F1m_mul
            end
            get_local $r
            get_local $r
            call $Fr_F1m_square
            get_local $b
            i32.const 16
            i32.ge_u
            if
                get_local $b
                i32.const 16
                i32.sub
                set_local $b
                i32.const 352
                get_local $r
                get_local $r
                call $Fr_F1m_mul
            end
            get_local $r
            get_local $r
            call $Fr_F1m_square
            get_local $b
            i32.const 8
            i32.ge_u
            if
                get_local $b
                i32.const 8
                i32.sub
                set_local $b
                i32.const 352
                get_local $r
                get_local $r
                call $Fr_F1m_mul
            end
            get_local $r
            get_local $r
            call $Fr_F1m_square
            get_local $b
            i32.const 4
            i32.ge_u
            if
                get_local $b
                i32.const 4
                i32.sub
                set_local $b
                i32.const 352
                get_local $r
                get_local $r
                call $Fr_F1m_mul
            end
            get_local $r
            get_local $r
            call $Fr_F1m_square
            get_local $b
            i32.const 2
            i32.ge_u
            if
                get_local $b
                i32.const 2
                i32.sub
                set_local $b
                i32.const 352
                get_local $r
                get_local $r
                call $Fr_F1m_mul
            end
            get_local $r
            get_local $r
            call $Fr_F1m_square
            get_local $b
            i32.const 1
            i32.ge_u
            if
                get_local $b
                i32.const 1
                i32.sub
                set_local $b
                i32.const 352
                get_local $r
                get_local $r
                call $Fr_F1m_mul
            end
            get_local $i
            i32.eqz
            br_if 1
            br 0
        end
    end
)
(func $Fr_F1m_sqrt (type $_sig_i32i32)
     (param $n i32)
     (param $r i32)
     (local $m i32)
     (local $i i32)
     (local $j i32)
    get_local $n
    call $Fr_int_isZero
    if
        get_local $r
        call $Fr_int_zero
        return
    end
    i32.const 32
    set_local $m
    i32.const 256
    i32.const 360
    call $Fr_int_copy
    get_local $n
    i32.const 248
    i32.const 8
    i32.const 368
    call $Fr_F1m_exp
    get_local $n
    i32.const 264
    i32.const 8
    i32.const 376
    call $Fr_F1m_exp
    block
        loop
            i32.const 368
            i32.const 208
            call $Fr_int_eq
            br_if 1
            i32.const 368
            i32.const 384
            call $Fr_F1m_square
            i32.const 1
            set_local $i
            block
                loop
                    i32.const 384
                    i32.const 208
                    call $Fr_int_eq
                    br_if 1
                    i32.const 384
                    i32.const 384
                    call $Fr_F1m_square
                    get_local $i
                    i32.const 1
                    i32.add
                    set_local $i
                    br 0
                end
            end
            i32.const 360
            i32.const 392
            call $Fr_int_copy
            get_local $m
            get_local $i
            i32.sub
            i32.const 1
            i32.sub
            set_local $j
            block
                loop
                    get_local $j
                    i32.eqz
                    br_if 1
                    i32.const 392
                    i32.const 392
                    call $Fr_F1m_square
                    get_local $j
                    i32.const 1
                    i32.sub
                    set_local $j
                    br 0
                end
            end
            get_local $i
            set_local $m
            i32.const 392
            i32.const 360
            call $Fr_F1m_square
            i32.const 368
            i32.const 360
            i32.const 368
            call $Fr_F1m_mul
            i32.const 376
            i32.const 392
            i32.const 376
            call $Fr_F1m_mul
            br 0
        end
    end
    i32.const 376
    call $Fr_F1m_isNegative
    if
        i32.const 376
        get_local $r
        call $Fr_F1m_neg
    else
        i32.const 376
        get_local $r
        call $Fr_int_copy
    end
)
(func $Fr_F1m_isSquare (type $_sig_i32ri32)
     (param $n i32)
    (result i32)
    get_local $n
    call $Fr_int_isZero
    if
        i32.const 1
        return
    end
    get_local $n
    i32.const 224
    i32.const 8
    i32.const 400
    call $Fr_F1m_exp
    i32.const 400
    i32.const 208
    call $Fr_int_eq
)
(func $Fr_copy (type $_sig_i32i32)
     (param $pr i32)
     (param $px i32)
    get_local $pr
    get_local $px
    i64.load
    i64.store
    get_local $pr
    get_local $px
    i64.load offset=8
    i64.store offset=8
)
(func $Fr_copyn (type $_sig_i32i32i32)
     (param $pr i32)
     (param $px i32)
     (param $n i32)
     (local $s i32)
     (local $d i32)
     (local $slast i32)
    get_local $px
    set_local $s
    get_local $pr
    set_local $d
    get_local $s
    get_local $n
    i32.const 16
    i32.mul
    i32.add
    set_local $slast
    block
        loop
            get_local $s
            get_local $slast
            i32.eq
            br_if 1
            get_local $d
            get_local $s
            i64.load
            i64.store
            get_local $d
            i32.const 8
            i32.add
            set_local $d
            get_local $s
            i32.const 8
            i32.add
            set_local $s
            br 0
        end
    end
)
(func $Fr_isTrue (type $_sig_i32ri32)
     (param $px i32)
    (result i32)
    get_local $px
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $px
        i32.const 8
        i32.add
        call $Fr_int_isZero ;; it was $Fr_F1m_isZero, but it does not exists
        i32.eqz
        return
    end
    get_local $px
    i32.load
    i32.const 0
    i32.ne
)
(func $Fr_rawCopyS2L (type $_sig_i32i64)
     (param $pR i32)
     (param $v i64)
    get_local $v
    i64.const 0
    i64.gt_s
    if
        get_local $pR
        get_local $v
        i64.store
    else
        i64.const 0
        get_local $v
        i64.sub
        set_local $v
        get_local $pR
        get_local $v
        i64.store
        get_local $pR
        get_local $pR
        call $Fr_F1m_neg
    end
)
(func $Fr_toMontgomery (type $_sig_i32)
     (param $pR i32)
    get_local $pR
    i32.load8_u offset=7
    i32.const 64
    i32.and
    if
        return
    else
        get_local $pR
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pR
            i32.const -1073741824
            i32.store offset=4
            get_local $pR
            i32.const 8
            i32.add
            get_local $pR
            i32.const 8
            i32.add
            call $Fr_F1m_toMontgomery
        else
            get_local $pR
            i32.const 8
            i32.add
            get_local $pR
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pR
            i32.const 8
            i32.add
            get_local $pR
            i32.const 8
            i32.add
            call $Fr_F1m_toMontgomery
            get_local $pR
            i32.const 1073741824
            i32.store offset=4
        end
    end
)
(func $Fr_toNormal (type $_sig_i32)
     (param $pR i32)
    get_local $pR
    i32.load8_u offset=7
    i32.const 64
    i32.and
    if
        get_local $pR
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
            get_local $pR
            i32.const 8
            i32.add
            get_local $pR
            i32.const 8
            i32.add
            call $Fr_F1m_fromMontgomery
        end
    end
)
(func $Fr_toLongNormal (type $_sig_i32)
     (param $pR i32)
    get_local $pR
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pR
        i32.load8_u offset=7
        i32.const 64
        i32.and
        if
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
            get_local $pR
            i32.const 8
            i32.add
            get_local $pR
            i32.const 8
            i32.add
            call $Fr_F1m_fromMontgomery
        end
    else
        get_local $pR
        i32.const 8
        i32.add
        get_local $pR
        i64.load32_s
        call $Fr_rawCopyS2L
        get_local $pR
        i32.const -2147483648
        i32.store offset=4
    end
)
(func $Fr_isNegative (type $_sig_i32ri32)
     (param $pA i32)
    (result i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pA
        call $Fr_toNormal
        get_local $pA
        i32.const 8
        i32.add
        i32.const 408
        call $Fr_int_gt
        return
    end
    get_local $pA
    i32.load
    i32.const 0
    i32.lt_s
)
(func $Fr_neg (type $_sig_i32i32)
     (param $pR i32)
     (param $pA i32)
     (local $r i64)
     (local $overflow i64)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pA
        i32.load8_u offset=7
        i32.const 64
        i32.and
        if
            get_local $pR
            i32.const -1073741824
            i32.store offset=4
        else
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pA
        i32.const 8
        i32.add
        get_local $pR
        i32.const 8
        i32.add
        call $Fr_F1m_neg
    else
        i64.const 0
        get_local $pA
        i64.load32_s
        i64.sub
        set_local $r
        get_local $r
        i64.const 31
        i64.shr_s
        set_local $overflow
        get_local $overflow
        i64.eqz
        get_local $overflow
        i64.const 1
        i64.add
        i64.eqz
        i32.or
        if
            get_local $pR
            get_local $r
            i64.store32
            get_local $pR
            i32.const 0
            i32.store offset=4
        else
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
            get_local $pR
            i32.const 8
            i32.add
            get_local $r
            call $Fr_rawCopyS2L
        end
    end
)
(func $Fr_getLsb32 (type $_sig_i32ri32)
     (param $pA i32)
    (result i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pA
        call $Fr_toNormal
        get_local $pA
        i32.load offset=8
        return
    else
        get_local $pA
        i32.load
        return
    end
    i32.const 0
)
(func $Fr_toInt (type $_sig_i32ri32)
     (param $pA i32)
    (result i32)
    get_local $pA
    call $Fr_isNegative
    if
        i32.const 8
        get_local $pA
        call $Fr_neg
        i32.const 0
        i32.const 8
        call $Fr_getLsb32
        i32.sub
        return
    else
        get_local $pA
        call $Fr_getLsb32
        return
    end
    i32.const 0
)
(func $Fr_add (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
     (local $r i64)
     (local $overflow i64)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pB
                i32.load8_u offset=7
                i32.const 64
                i32.and
                if
                    get_local $pR
                    i32.const -1073741824
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_add
                else
                    get_local $pB
                    call $Fr_toMontgomery
                    get_local $pR
                    i32.const -1073741824
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_add
                end
            else
                get_local $pB
                i32.load8_u offset=7
                i32.const 64
                i32.and
                if
                    get_local $pA
                    call $Fr_toMontgomery
                    get_local $pR
                    i32.const -1073741824
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_add
                else
                    get_local $pR
                    i32.const -2147483648
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_add
                end
            end
        else
            get_local $pA
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pB
                call $Fr_toMontgomery
                get_local $pR
                i32.const -1073741824
                i32.store offset=4
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_add
            else
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                i32.const 16
                get_local $pB
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const 8
                i32.add
                i32.const 16
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_add
            end
        end
    else
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pB
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pA
                call $Fr_toMontgomery
                get_local $pR
                i32.const -1073741824
                i32.store offset=4
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_add
            else
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                i32.const 16
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                i32.const 16
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_add
            end
        else
            get_local $pA
            i64.load32_s
            get_local $pB
            i64.load32_s
            i64.add
            set_local $r
            get_local $r
            i64.const 31
            i64.shr_s
            set_local $overflow
            get_local $overflow
            i64.eqz
            get_local $overflow
            i64.const 1
            i64.add
            i64.eqz
            i32.or
            if
                get_local $pR
                get_local $r
                i64.store32
                get_local $pR
                i32.const 0
                i32.store offset=4
            else
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                get_local $pR
                i32.const 8
                i32.add
                get_local $r
                call $Fr_rawCopyS2L
            end
        end
    end
)
(func $Fr_sub (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
     (local $r i64)
     (local $overflow i64)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pB
                i32.load8_u offset=7
                i32.const 64
                i32.and
                if
                    get_local $pR
                    i32.const -1073741824
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_sub
                else
                    get_local $pB
                    call $Fr_toMontgomery
                    get_local $pR
                    i32.const -1073741824
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_sub
                end
            else
                get_local $pB
                i32.load8_u offset=7
                i32.const 64
                i32.and
                if
                    get_local $pA
                    call $Fr_toMontgomery
                    get_local $pR
                    i32.const -1073741824
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_sub
                else
                    get_local $pR
                    i32.const -2147483648
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_sub
                end
            end
        else
            get_local $pA
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pB
                call $Fr_toMontgomery
                get_local $pR
                i32.const -1073741824
                i32.store offset=4
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_sub
            else
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                i32.const 16
                get_local $pB
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const 8
                i32.add
                i32.const 16
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_sub
            end
        end
    else
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pB
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pA
                call $Fr_toMontgomery
                get_local $pR
                i32.const -1073741824
                i32.store offset=4
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_sub
            else
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                i32.const 16
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                i32.const 16
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_sub
            end
        else
            get_local $pA
            i64.load32_s
            get_local $pB
            i64.load32_s
            i64.sub
            set_local $r
            get_local $r
            i64.const 31
            i64.shr_s
            set_local $overflow
            get_local $overflow
            i64.eqz
            get_local $overflow
            i64.const 1
            i64.add
            i64.eqz
            i32.or
            if
                get_local $pR
                get_local $r
                i64.store32
                get_local $pR
                i32.const 0
                i32.store offset=4
            else
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                get_local $pR
                i32.const 8
                i32.add
                get_local $r
                call $Fr_rawCopyS2L
            end
        end
    end
)
(func $Fr_eqR (type $_sig_i32i32ri32)
     (param $pA i32)
     (param $pB i32)
    (result i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
        else
            get_local $pB
            i32.const 8
            i32.add
            get_local $pB
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pB
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pA
        i32.load8_u offset=7
        i32.const 64
        i32.and
        if
            get_local $pB
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                call $Fr_int_eq
                if
                    i32.const 1
                    return
                else
                    i32.const 0
                    return
                end
            else
                get_local $pA
                call $Fr_toNormal
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                call $Fr_int_eq
                if
                    i32.const 1
                    return
                else
                    i32.const 0
                    return
                end
            end
        else
            get_local $pB
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pB
                call $Fr_toNormal
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                call $Fr_int_eq
                if
                    i32.const 1
                    return
                else
                    i32.const 0
                    return
                end
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                call $Fr_int_eq
                if
                    i32.const 1
                    return
                else
                    i32.const 0
                    return
                end
            end
        end
    else
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pA
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pB
                i32.load8_u offset=7
                i32.const 64
                i32.and
                if
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    call $Fr_int_eq
                    if
                        i32.const 1
                        return
                    else
                        i32.const 0
                        return
                    end
                else
                    get_local $pA
                    call $Fr_toNormal
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    call $Fr_int_eq
                    if
                        i32.const 1
                        return
                    else
                        i32.const 0
                        return
                    end
                end
            else
                get_local $pB
                i32.load8_u offset=7
                i32.const 64
                i32.and
                if
                    get_local $pB
                    call $Fr_toNormal
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    call $Fr_int_eq
                    if
                        i32.const 1
                        return
                    else
                        i32.const 0
                        return
                    end
                else
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    call $Fr_int_eq
                    if
                        i32.const 1
                        return
                    else
                        i32.const 0
                        return
                    end
                end
            end
        else
            get_local $pA
            i32.load
            get_local $pB
            i32.load
            i32.eq
            if
                i32.const 1
                return
            else
                i32.const 0
                return
            end
        end
    end
    i32.const 0
)
(func $Fr_gtR (type $_sig_i32i32ri32)
     (param $pA i32)
     (param $pB i32)
    (result i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
        else
            get_local $pB
            i32.const 8
            i32.add
            get_local $pB
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pB
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pA
        call $Fr_toNormal
        get_local $pB
        call $Fr_toNormal
        get_local $pA
        call $Fr_isNegative
        if
            get_local $pB
            call $Fr_isNegative
            if
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                call $Fr_int_gt
                if
                    i32.const 1
                    return
                else
                    i32.const 0
                    return
                end
            else
                i32.const 0
                return
            end
        else
            get_local $pB
            call $Fr_isNegative
            if
                i32.const 1
                return
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                call $Fr_int_gt
                if
                    i32.const 1
                    return
                else
                    i32.const 0
                    return
                end
            end
        end
    else
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pA
            call $Fr_toNormal
            get_local $pB
            call $Fr_toNormal
            get_local $pA
            call $Fr_isNegative
            if
                get_local $pB
                call $Fr_isNegative
                if
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    call $Fr_int_gt
                    if
                        i32.const 1
                        return
                    else
                        i32.const 0
                        return
                    end
                else
                    i32.const 0
                    return
                end
            else
                get_local $pB
                call $Fr_isNegative
                if
                    i32.const 1
                    return
                else
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    call $Fr_int_gt
                    if
                        i32.const 1
                        return
                    else
                        i32.const 0
                        return
                    end
                end
            end
        else
            get_local $pA
            i32.load
            get_local $pB
            i32.load
            i32.gt_s
            if
                i32.const 1
                return
            else
                i32.const 0
                return
            end
        end
    end
    i32.const 0
)
(func $Fr_eq (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    get_local $pB
    call $Fr_eqR
    if
        get_local $pR
        i64.const 1
        i64.store
    else
        get_local $pR
        i64.const 0
        i64.store
    end
)
(func $Fr_neq (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    get_local $pB
    call $Fr_eqR
    if
        get_local $pR
        i64.const 0
        i64.store
    else
        get_local $pR
        i64.const 1
        i64.store
    end
)
(func $Fr_gt (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    get_local $pB
    call $Fr_eqR
    if
        get_local $pR
        i64.const 0
        i64.store
    else
        get_local $pA
        get_local $pB
        call $Fr_gtR
        if
            get_local $pR
            i64.const 1
            i64.store
        else
            get_local $pR
            i64.const 0
            i64.store
        end
    end
)
(func $Fr_geq (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    get_local $pB
    call $Fr_eqR
    if
        get_local $pR
        i64.const 1
        i64.store
    else
        get_local $pA
        get_local $pB
        call $Fr_gtR
        if
            get_local $pR
            i64.const 1
            i64.store
        else
            get_local $pR
            i64.const 0
            i64.store
        end
    end
)
(func $Fr_lt (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    get_local $pB
    call $Fr_eqR
    if
        get_local $pR
        i64.const 0
        i64.store
    else
        get_local $pA
        get_local $pB
        call $Fr_gtR
        if
            get_local $pR
            i64.const 0
            i64.store
        else
            get_local $pR
            i64.const 1
            i64.store
        end
    end
)
(func $Fr_leq (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    get_local $pB
    call $Fr_eqR
    if
        get_local $pR
        i64.const 1
        i64.store
    else
        get_local $pA
        get_local $pB
        call $Fr_gtR
        if
            get_local $pR
            i64.const 0
            i64.store
        else
            get_local $pR
            i64.const 1
            i64.store
        end
    end
)
(func $Fr_mul (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
     (local $r i64)
     (local $overflow i64)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pB
                i32.load8_u offset=7
                i32.const 64
                i32.and
                if
                    get_local $pR
                    i32.const -1073741824
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_mul
                else
                    get_local $pR
                    i32.const -2147483648
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_mul
                end
            else
                get_local $pB
                i32.load8_u offset=7
                i32.const 64
                i32.and
                if
                    get_local $pR
                    i32.const -2147483648
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_mul
                else
                    get_local $pR
                    i32.const -1073741824
                    i32.store offset=4
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_mul
                    i32.const 200
                    get_local $pR
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_F1m_mul
                end
            end
        else
            get_local $pA
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pB
                call $Fr_toMontgomery
                get_local $pR
                i32.const -1073741824
                i32.store offset=4
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_mul
            else
                get_local $pB
                call $Fr_toMontgomery
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_mul
            end
        end
    else
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
            get_local $pB
            i32.load8_u offset=7
            i32.const 64
            i32.and
            if
                get_local $pA
                call $Fr_toMontgomery
                get_local $pR
                i32.const -1073741824
                i32.store offset=4
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_mul
            else
                get_local $pA
                call $Fr_toMontgomery
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_F1m_mul
            end
        else
            get_local $pA
            i64.load32_s
            get_local $pB
            i64.load32_s
            i64.mul
            set_local $r
            get_local $r
            i64.const 31
            i64.shr_s
            set_local $overflow
            get_local $overflow
            i64.eqz
            get_local $overflow
            i64.const 1
            i64.add
            i64.eqz
            i32.or
            if
                get_local $pR
                get_local $r
                i64.store32
                get_local $pR
                i32.const 0
                i32.store offset=4
            else
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                get_local $pR
                i32.const 8
                i32.add
                get_local $r
                call $Fr_rawCopyS2L
            end
        end
    end
)
(func $Fr_idiv (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
    else
        get_local $pA
        i32.const 8
        i32.add
        get_local $pA
        i64.load32_s
        call $Fr_rawCopyS2L
        get_local $pA
        i32.const -2147483648
        i32.store offset=4
    end
    get_local $pA
    call $Fr_toNormal
    get_local $pB
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
    else
        get_local $pB
        i32.const 8
        i32.add
        get_local $pB
        i64.load32_s
        call $Fr_rawCopyS2L
        get_local $pB
        i32.const -2147483648
        i32.store offset=4
    end
    get_local $pB
    call $Fr_toNormal
    get_local $pR
    i32.const -2147483648
    i32.store offset=4
    get_local $pA
    i32.const 8
    i32.add
    get_local $pB
    i32.const 8
    i32.add
    get_local $pR
    i32.const 8
    i32.add
    i32.const 16
    call $Fr_int_div
)
(func $Fr_mod (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
    else
        get_local $pA
        i32.const 8
        i32.add
        get_local $pA
        i64.load32_s
        call $Fr_rawCopyS2L
        get_local $pA
        i32.const -2147483648
        i32.store offset=4
    end
    get_local $pA
    call $Fr_toNormal
    get_local $pB
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
    else
        get_local $pB
        i32.const 8
        i32.add
        get_local $pB
        i64.load32_s
        call $Fr_rawCopyS2L
        get_local $pB
        i32.const -2147483648
        i32.store offset=4
    end
    get_local $pB
    call $Fr_toNormal
    get_local $pR
    i32.const -2147483648
    i32.store offset=4
    get_local $pA
    i32.const 8
    i32.add
    get_local $pB
    i32.const 8
    i32.add
    i32.const 16
    get_local $pR
    i32.const 8
    i32.add
    call $Fr_int_div
)
(func $Fr_inv (type $_sig_i32i32)
     (param $pR i32)
     (param $pA i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
    else
        get_local $pA
        i32.const 8
        i32.add
        get_local $pA
        i64.load32_s
        call $Fr_rawCopyS2L
        get_local $pA
        i32.const -2147483648
        i32.store offset=4
    end
    get_local $pA
    i32.const 8
    i32.add
    i32.const 176
    get_local $pR
    i32.const 8
    i32.add
    call $Fr_int_inverseMod
    get_local $pA
    i32.load8_u offset=7
    i32.const 64
    i32.and
    if
        get_local $pR
        i32.const -1073741824
        i32.store offset=4
        get_local $pR
        i32.const 8
        i32.add
        i32.const 200
        get_local $pR
        i32.const 8
        i32.add
        call $Fr_F1m_mul
    else
        get_local $pR
        i32.const -2147483648
        i32.store offset=4
    end
)
(func $Fr_div (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
     (local $r i64)
     (local $overflow i64)
    get_local $pR
    get_local $pB
    call $Fr_inv
    get_local $pR
    get_local $pR
    get_local $pA
    call $Fr_mul
)
(func $Fr_pow (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    call $Fr_toMontgomery
    get_local $pB
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
    else
        get_local $pB
        i32.const 8
        i32.add
        get_local $pB
        i64.load32_s
        call $Fr_rawCopyS2L
        get_local $pB
        i32.const -2147483648
        i32.store offset=4
    end
    get_local $pB
    call $Fr_toNormal
    get_local $pR
    i32.const -1073741824
    i32.store offset=4
    get_local $pA
    i32.const 8
    i32.add
    get_local $pB
    i32.const 8
    i32.add
    i32.const 8
    get_local $pR
    i32.const 8
    i32.add
    call $Fr_F1m_exp
)
(func $Fr_fixedShl (type $_sig_i64i64ri64)
     (param $a i64)
     (param $b i64)
    (result i64)
    get_local $b
    i64.const 64
    i64.ge_u
    if
        i64.const 0
        return
    end
    get_local $a
    get_local $b
    i64.shl
)
(func $Fr_fixedShr (type $_sig_i64i64ri64)
     (param $a i64)
     (param $b i64)
    (result i64)
    get_local $b
    i64.const 64
    i64.ge_u
    if
        i64.const 0
        return
    end
    get_local $a
    get_local $b
    i64.shr_u
)
(func $Fr_rawgetchunk (type $_sig_i32i32ri64)
     (param $pA i32)
     (param $i i32)
    (result i64)
    get_local $i
    i32.const 1
    i32.lt_u
    if
        get_local $pA
        get_local $i
        i32.const 8
        i32.mul
        i32.add
        i64.load
        return
    end
    i64.const 0
)
(func $Fr_rawshll (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $n i32)
     (local $oWords1 i32)
     (local $oBits1 i64)
     (local $oWords2 i32)
     (local $oBits2 i64)
     (local $i i32)
    i32.const 0
    get_local $n
    i32.const 6
    i32.shr_u
    i32.sub
    set_local $oWords1
    get_local $oWords1
    i32.const 1
    i32.sub
    set_local $oWords2
    get_local $n
    i64.extend_u/i32
    i64.const 63
    i64.and
    set_local $oBits1
    i64.const 64
    get_local $oBits1
    i64.sub
    set_local $oBits2
    i32.const 0
    set_local $i
    block
        loop
            get_local $i
            i32.const 1
            i32.eq
            br_if 1
            get_local $pR
            get_local $i
            i32.const 8
            i32.mul
            i32.add
            get_local $pA
            get_local $oWords1
            get_local $i
            i32.add
            call $Fr_rawgetchunk
            get_local $oBits1
            call $Fr_fixedShl
            get_local $pA
            get_local $oWords2
            get_local $i
            i32.add
            call $Fr_rawgetchunk
            get_local $oBits2
            call $Fr_fixedShr
            i64.or
            i64.store
            get_local $i
            i32.const 1
            i32.add
            set_local $i
            br 0
        end
    end
)
(func $Fr_rawshrl (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $n i32)
     (local $oWords1 i32)
     (local $oBits1 i64)
     (local $oWords2 i32)
     (local $oBits2 i64)
     (local $i i32)
    get_local $n
    i32.const 6
    i32.shr_u
    set_local $oWords1
    get_local $oWords1
    i32.const 1
    i32.add
    set_local $oWords2
    get_local $n
    i64.extend_u/i32
    i64.const 63
    i64.and
    set_local $oBits1
    i64.const 64
    get_local $oBits1
    i64.sub
    set_local $oBits2
    i32.const 0
    set_local $i
    block
        loop
            get_local $i
            i32.const 1
            i32.eq
            br_if 1
            get_local $pR
            get_local $i
            i32.const 8
            i32.mul
            i32.add
            get_local $pA
            get_local $oWords1
            get_local $i
            i32.add
            call $Fr_rawgetchunk
            get_local $oBits1
            call $Fr_fixedShr
            get_local $pA
            get_local $oWords2
            get_local $i
            i32.add
            call $Fr_rawgetchunk
            get_local $oBits2
            call $Fr_fixedShl
            i64.or
            i64.store
            get_local $i
            i32.const 1
            i32.add
            set_local $i
            br 0
        end
    end
)
(func $Fr_adjustBinResult (type $_sig_i32)
     (param $pA i32)
    get_local $pA
    get_local $pA
    i64.load offset=8
    i64.const 18446744073709551615
    i64.and
    i64.store offset=8
    get_local $pA
    i32.const 8
    i32.add
    i32.const 176
    call $Fr_int_gte
    if
        get_local $pA
        i32.const 8
        i32.add
        i32.const 176
        get_local $pA
        i32.const 8
        i32.add
        call $Fr_int_sub
        drop
    end
)
(func $Fr_rawshl (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $n i32)
     (local $r i64)
     (local $overflow i64)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pA
        call $Fr_toNormal
        get_local $pR
        i32.const 8
        i32.add
        get_local $pA
        i32.const 8
        i32.add
        get_local $n
        call $Fr_rawshll
        get_local $pR
        call $Fr_adjustBinResult
        get_local $pR
        i32.const -2147483648
        i32.store offset=4
    else
        get_local $pA
        call $Fr_isNegative
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pR
            i32.const 8
            i32.add
            get_local $pA
            i32.const 8
            i32.add
            get_local $n
            call $Fr_rawshll
            get_local $pR
            call $Fr_adjustBinResult
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
        else
            get_local $n
            i32.const 30
            i32.gt_u
            if
                get_local $pA
                i32.load8_u offset=7
                i32.const 128
                i32.and
                if
                else
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pA
                    i64.load32_s
                    call $Fr_rawCopyS2L
                    get_local $pA
                    i32.const -2147483648
                    i32.store offset=4
                end
                get_local $pR
                i32.const 8
                i32.add
                get_local $pA
                i32.const 8
                i32.add
                get_local $n
                call $Fr_rawshll
                get_local $pR
                call $Fr_adjustBinResult
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
            else
                get_local $pA
                i64.load32_s
                get_local $n
                i64.extend_u/i32
                i64.shl
                set_local $r
                get_local $r
                i64.const 31
                i64.shr_s
                set_local $overflow
                get_local $overflow
                i64.eqz
                get_local $overflow
                i64.const 1
                i64.add
                i64.eqz
                i32.or
                if
                    get_local $pR
                    get_local $r
                    i64.store32
                    get_local $pR
                    i32.const 0
                    i32.store offset=4
                else
                    get_local $pR
                    i32.const -2147483648
                    i32.store offset=4
                    get_local $pR
                    i32.const 8
                    i32.add
                    get_local $r
                    call $Fr_rawCopyS2L
                end
            end
        end
    end
)
(func $Fr_rawshr (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $n i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pA
        call $Fr_toNormal
        get_local $pR
        i32.const 8
        i32.add
        get_local $pA
        i32.const 8
        i32.add
        get_local $n
        call $Fr_rawshrl
        get_local $pR
        i32.const -2147483648
        i32.store offset=4
    else
        get_local $pA
        call $Fr_isNegative
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pR
            i32.const 8
            i32.add
            get_local $pA
            i32.const 8
            i32.add
            get_local $n
            call $Fr_rawshrl
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
        else
            get_local $n
            i32.const 32
            i32.lt_u
            if
                get_local $pR
                get_local $pA
                i32.load
                get_local $n
                i32.shr_u
                i32.store
            else
                get_local $pR
                i32.const 0
                i32.store
            end
            get_local $pR
            i32.const 0
            i32.store offset=4
        end
    end
)
(func $Fr_shl (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pB
    call $Fr_isNegative
    if
        i32.const 24
        get_local $pB
        call $Fr_neg
        i32.const 8
        i32.const 24
        i32.const 40
        call $Fr_lt
        i32.const 8
        i32.load
        if
            get_local $pR
            get_local $pA
            i32.const 24
            call $Fr_toInt
            call $Fr_rawshr
        else
            get_local $pR
            call $Fr_int_zero
        end
    else
        i32.const 8
        get_local $pB
        i32.const 40
        call $Fr_lt
        i32.const 8
        i32.load
        if
            get_local $pR
            get_local $pA
            get_local $pB
            call $Fr_toInt
            call $Fr_rawshl
        else
            get_local $pR
            call $Fr_int_zero
        end
    end
)
(func $Fr_shr (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pB
    call $Fr_isNegative
    if
        i32.const 24
        get_local $pB
        call $Fr_neg
        i32.const 8
        i32.const 24
        i32.const 40
        call $Fr_lt
        i32.const 8
        i32.load
        if
            get_local $pR
            get_local $pA
            i32.const 24
            call $Fr_toInt
            call $Fr_rawshl
        else
            get_local $pR
            call $Fr_int_zero
        end
    else
        i32.const 8
        get_local $pB
        i32.const 40
        call $Fr_lt
        i32.const 8
        i32.load
        if
            get_local $pR
            get_local $pA
            get_local $pB
            call $Fr_toInt
            call $Fr_rawshr
        else
            get_local $pR
            call $Fr_int_zero
        end
    end
)
(func $Fr_rawbandl (type $_sig_i32i32i32)
     (param $pA i32)
     (param $pB i32)
     (param $pR i32)
    get_local $pR
    get_local $pA
    i64.load
    get_local $pB
    i64.load
    i64.and
    i64.store
)
(func $Fr_band (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pA
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
        else
            get_local $pA
            i32.const 8
            i32.add
            get_local $pA
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pA
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pA
        call $Fr_toNormal
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
        else
            get_local $pB
            i32.const 8
            i32.add
            get_local $pB
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pB
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pB
        call $Fr_toNormal
        get_local $pA
        i32.const 8
        i32.add
        get_local $pB
        i32.const 8
        i32.add
        get_local $pR
        i32.const 8
        i32.add
        call $Fr_rawbandl
        get_local $pR
        i32.const -2147483648
        i32.store offset=4
        get_local $pR
        call $Fr_adjustBinResult
    else
        get_local $pA
        call $Fr_isNegative
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pA
            call $Fr_toNormal
            get_local $pB
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pB
                i32.const 8
                i32.add
                get_local $pB
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pB
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pB
            call $Fr_toNormal
            get_local $pA
            i32.const 8
            i32.add
            get_local $pB
            i32.const 8
            i32.add
            get_local $pR
            i32.const 8
            i32.add
            call $Fr_rawbandl
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
            get_local $pR
            call $Fr_adjustBinResult
        else
            get_local $pB
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
                get_local $pA
                i32.load8_u offset=7
                i32.const 128
                i32.and
                if
                else
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pA
                    i64.load32_s
                    call $Fr_rawCopyS2L
                    get_local $pA
                    i32.const -2147483648
                    i32.store offset=4
                end
                get_local $pA
                call $Fr_toNormal
                get_local $pB
                i32.load8_u offset=7
                i32.const 128
                i32.and
                if
                else
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pB
                    i64.load32_s
                    call $Fr_rawCopyS2L
                    get_local $pB
                    i32.const -2147483648
                    i32.store offset=4
                end
                get_local $pB
                call $Fr_toNormal
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_rawbandl
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                get_local $pR
                call $Fr_adjustBinResult
            else
                get_local $pB
                call $Fr_isNegative
                if
                    get_local $pA
                    i32.load8_u offset=7
                    i32.const 128
                    i32.and
                    if
                    else
                        get_local $pA
                        i32.const 8
                        i32.add
                        get_local $pA
                        i64.load32_s
                        call $Fr_rawCopyS2L
                        get_local $pA
                        i32.const -2147483648
                        i32.store offset=4
                    end
                    get_local $pA
                    call $Fr_toNormal
                    get_local $pB
                    i32.load8_u offset=7
                    i32.const 128
                    i32.and
                    if
                    else
                        get_local $pB
                        i32.const 8
                        i32.add
                        get_local $pB
                        i64.load32_s
                        call $Fr_rawCopyS2L
                        get_local $pB
                        i32.const -2147483648
                        i32.store offset=4
                    end
                    get_local $pB
                    call $Fr_toNormal
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_rawbandl
                    get_local $pR
                    i32.const -2147483648
                    i32.store offset=4
                    get_local $pR
                    call $Fr_adjustBinResult
                else
                    get_local $pR
                    get_local $pA
                    i32.load
                    get_local $pB
                    i32.load
                    i32.and
                    i32.store
                    get_local $pR
                    i32.const 0
                    i32.store offset=4
                end
            end
        end
    end
)
(func $Fr_rawborl (type $_sig_i32i32i32)
     (param $pA i32)
     (param $pB i32)
     (param $pR i32)
    get_local $pR
    get_local $pA
    i64.load
    get_local $pB
    i64.load
    i64.or
    i64.store
)
(func $Fr_bor (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pA
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
        else
            get_local $pA
            i32.const 8
            i32.add
            get_local $pA
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pA
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pA
        call $Fr_toNormal
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
        else
            get_local $pB
            i32.const 8
            i32.add
            get_local $pB
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pB
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pB
        call $Fr_toNormal
        get_local $pA
        i32.const 8
        i32.add
        get_local $pB
        i32.const 8
        i32.add
        get_local $pR
        i32.const 8
        i32.add
        call $Fr_rawborl
        get_local $pR
        i32.const -2147483648
        i32.store offset=4
        get_local $pR
        call $Fr_adjustBinResult
    else
        get_local $pA
        call $Fr_isNegative
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pA
            call $Fr_toNormal
            get_local $pB
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pB
                i32.const 8
                i32.add
                get_local $pB
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pB
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pB
            call $Fr_toNormal
            get_local $pA
            i32.const 8
            i32.add
            get_local $pB
            i32.const 8
            i32.add
            get_local $pR
            i32.const 8
            i32.add
            call $Fr_rawborl
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
            get_local $pR
            call $Fr_adjustBinResult
        else
            get_local $pB
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
                get_local $pA
                i32.load8_u offset=7
                i32.const 128
                i32.and
                if
                else
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pA
                    i64.load32_s
                    call $Fr_rawCopyS2L
                    get_local $pA
                    i32.const -2147483648
                    i32.store offset=4
                end
                get_local $pA
                call $Fr_toNormal
                get_local $pB
                i32.load8_u offset=7
                i32.const 128
                i32.and
                if
                else
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pB
                    i64.load32_s
                    call $Fr_rawCopyS2L
                    get_local $pB
                    i32.const -2147483648
                    i32.store offset=4
                end
                get_local $pB
                call $Fr_toNormal
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_rawborl
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                get_local $pR
                call $Fr_adjustBinResult
            else
                get_local $pB
                call $Fr_isNegative
                if
                    get_local $pA
                    i32.load8_u offset=7
                    i32.const 128
                    i32.and
                    if
                    else
                        get_local $pA
                        i32.const 8
                        i32.add
                        get_local $pA
                        i64.load32_s
                        call $Fr_rawCopyS2L
                        get_local $pA
                        i32.const -2147483648
                        i32.store offset=4
                    end
                    get_local $pA
                    call $Fr_toNormal
                    get_local $pB
                    i32.load8_u offset=7
                    i32.const 128
                    i32.and
                    if
                    else
                        get_local $pB
                        i32.const 8
                        i32.add
                        get_local $pB
                        i64.load32_s
                        call $Fr_rawCopyS2L
                        get_local $pB
                        i32.const -2147483648
                        i32.store offset=4
                    end
                    get_local $pB
                    call $Fr_toNormal
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_rawborl
                    get_local $pR
                    i32.const -2147483648
                    i32.store offset=4
                    get_local $pR
                    call $Fr_adjustBinResult
                else
                    get_local $pR
                    get_local $pA
                    i32.load
                    get_local $pB
                    i32.load
                    i32.or
                    i32.store
                    get_local $pR
                    i32.const 0
                    i32.store offset=4
                end
            end
        end
    end
)
(func $Fr_rawbxorl (type $_sig_i32i32i32)
     (param $pA i32)
     (param $pB i32)
     (param $pR i32)
    get_local $pR
    get_local $pA
    i64.load
    get_local $pB
    i64.load
    i64.xor
    i64.store
)
(func $Fr_bxor (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
        get_local $pA
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
        else
            get_local $pA
            i32.const 8
            i32.add
            get_local $pA
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pA
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pA
        call $Fr_toNormal
        get_local $pB
        i32.load8_u offset=7
        i32.const 128
        i32.and
        if
        else
            get_local $pB
            i32.const 8
            i32.add
            get_local $pB
            i64.load32_s
            call $Fr_rawCopyS2L
            get_local $pB
            i32.const -2147483648
            i32.store offset=4
        end
        get_local $pB
        call $Fr_toNormal
        get_local $pA
        i32.const 8
        i32.add
        get_local $pB
        i32.const 8
        i32.add
        get_local $pR
        i32.const 8
        i32.add
        call $Fr_rawbxorl
        get_local $pR
        i32.const -2147483648
        i32.store offset=4
        get_local $pR
        call $Fr_adjustBinResult
    else
        get_local $pA
        call $Fr_isNegative
        if
            get_local $pA
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pA
                i32.const 8
                i32.add
                get_local $pA
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pA
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pA
            call $Fr_toNormal
            get_local $pB
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
            else
                get_local $pB
                i32.const 8
                i32.add
                get_local $pB
                i64.load32_s
                call $Fr_rawCopyS2L
                get_local $pB
                i32.const -2147483648
                i32.store offset=4
            end
            get_local $pB
            call $Fr_toNormal
            get_local $pA
            i32.const 8
            i32.add
            get_local $pB
            i32.const 8
            i32.add
            get_local $pR
            i32.const 8
            i32.add
            call $Fr_rawbxorl
            get_local $pR
            i32.const -2147483648
            i32.store offset=4
            get_local $pR
            call $Fr_adjustBinResult
        else
            get_local $pB
            i32.load8_u offset=7
            i32.const 128
            i32.and
            if
                get_local $pA
                i32.load8_u offset=7
                i32.const 128
                i32.and
                if
                else
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pA
                    i64.load32_s
                    call $Fr_rawCopyS2L
                    get_local $pA
                    i32.const -2147483648
                    i32.store offset=4
                end
                get_local $pA
                call $Fr_toNormal
                get_local $pB
                i32.load8_u offset=7
                i32.const 128
                i32.and
                if
                else
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pB
                    i64.load32_s
                    call $Fr_rawCopyS2L
                    get_local $pB
                    i32.const -2147483648
                    i32.store offset=4
                end
                get_local $pB
                call $Fr_toNormal
                get_local $pA
                i32.const 8
                i32.add
                get_local $pB
                i32.const 8
                i32.add
                get_local $pR
                i32.const 8
                i32.add
                call $Fr_rawbxorl
                get_local $pR
                i32.const -2147483648
                i32.store offset=4
                get_local $pR
                call $Fr_adjustBinResult
            else
                get_local $pB
                call $Fr_isNegative
                if
                    get_local $pA
                    i32.load8_u offset=7
                    i32.const 128
                    i32.and
                    if
                    else
                        get_local $pA
                        i32.const 8
                        i32.add
                        get_local $pA
                        i64.load32_s
                        call $Fr_rawCopyS2L
                        get_local $pA
                        i32.const -2147483648
                        i32.store offset=4
                    end
                    get_local $pA
                    call $Fr_toNormal
                    get_local $pB
                    i32.load8_u offset=7
                    i32.const 128
                    i32.and
                    if
                    else
                        get_local $pB
                        i32.const 8
                        i32.add
                        get_local $pB
                        i64.load32_s
                        call $Fr_rawCopyS2L
                        get_local $pB
                        i32.const -2147483648
                        i32.store offset=4
                    end
                    get_local $pB
                    call $Fr_toNormal
                    get_local $pA
                    i32.const 8
                    i32.add
                    get_local $pB
                    i32.const 8
                    i32.add
                    get_local $pR
                    i32.const 8
                    i32.add
                    call $Fr_rawbxorl
                    get_local $pR
                    i32.const -2147483648
                    i32.store offset=4
                    get_local $pR
                    call $Fr_adjustBinResult
                else
                    get_local $pR
                    get_local $pA
                    i32.load
                    get_local $pB
                    i32.load
                    i32.xor
                    i32.store
                    get_local $pR
                    i32.const 0
                    i32.store offset=4
                end
            end
        end
    end
)
(func $Fr_rawbnotl (type $_sig_i32i32)
     (param $pA i32)
     (param $pR i32)
    get_local $pR
    get_local $pA
    i64.load
    i64.const -1
    i64.xor
    i64.store
)
(func $Fr_bnot (type $_sig_i32i32)
     (param $pR i32)
     (param $pA i32)
    get_local $pA
    i32.load8_u offset=7
    i32.const 128
    i32.and
    if
    else
        get_local $pA
        i32.const 8
        i32.add
        get_local $pA
        i64.load32_s
        call $Fr_rawCopyS2L
        get_local $pA
        i32.const -2147483648
        i32.store offset=4
    end
    get_local $pA
    call $Fr_toNormal
    get_local $pA
    i32.const 8
    i32.add
    get_local $pR
    i32.const 8
    i32.add
    call $Fr_rawbnotl
    get_local $pR
    i32.const -2147483648
    i32.store offset=4
    get_local $pR
    call $Fr_adjustBinResult
)
(func $Fr_land (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    call $Fr_isTrue
    get_local $pB
    call $Fr_isTrue
    i32.and
    if
        get_local $pR
        i64.const 1
        i64.store
    else
        get_local $pR
        i64.const 0
        i64.store
    end
)
(func $Fr_lor (type $_sig_i32i32i32)
     (param $pR i32)
     (param $pA i32)
     (param $pB i32)
    get_local $pA
    call $Fr_isTrue
    get_local $pB
    call $Fr_isTrue
    i32.or
    if
        get_local $pR
        i64.const 1
        i64.store
    else
        get_local $pR
        i64.const 0
        i64.store
    end
)
(func $Fr_lnot (type $_sig_i32i32)
     (param $pR i32)
     (param $pA i32)
    get_local $pA
    call $Fr_isTrue
    if
        get_local $pR
        i64.const 0
        i64.store
    else
        get_local $pR
        i64.const 1
        i64.store
    end
)
