

        global Fr_copy
        global Fr_copyn
        global Fr_add
        global Fr_sub
        global Fr_neg
        global Fr_mul
        global Fr_square
        global Fr_band
        global Fr_bor
        global Fr_bxor
        global Fr_bnot
        global Fr_shl
        global Fr_shr
        global Fr_eq
        global Fr_neq
        global Fr_lt
        global Fr_gt
        global Fr_leq
        global Fr_geq
        global Fr_land
        global Fr_lor
        global Fr_lnot
        global Fr_toNormal
        global Fr_toLongNormal
        global Fr_toMontgomery
        global Fr_toInt
        global Fr_isTrue
        global Fr_q
        global Fr_R3

        global Fr_rawCopy
        global Fr_rawZero
        global Fr_rawSwap
        global Fr_rawAdd
        global Fr_rawSub
        global Fr_rawNeg
        global Fr_rawMMul
        global Fr_rawMSquare
        global Fr_rawToMontgomery
        global Fr_rawFromMontgomery
        global Fr_rawIsEq
        global Fr_rawIsZero
        global Fr_rawq
        global Fr_rawR3

        extern Fr_fail
        DEFAULT REL

        section .text
















;;;;;;;;;;;;;;;;;;;;;;
; copy
;;;;;;;;;;;;;;;;;;;;;;
; Copies
; Params:
;   rsi <= the src
;   rdi <= the dest
;
; Nidified registers:
;   rax
;;;;;;;;;;;;;;;;;;;;;;;
Fr_copy:

        mov     rax, [rsi + 0]
        mov     [rdi + 0], rax

        mov     rax, [rsi + 8]
        mov     [rdi + 8], rax

        ret


;;;;;;;;;;;;;;;;;;;;;;
; rawCopy
;;;;;;;;;;;;;;;;;;;;;;
; Copies
; Params:
;   rsi <= the src
;   rdi <= the dest
;
; Nidified registers:
;   rax
;;;;;;;;;;;;;;;;;;;;;;;
Fr_rawCopy:

        mov     rax, [rsi + 0]
        mov     [rdi + 0], rax

        ret


;;;;;;;;;;;;;;;;;;;;;;
; rawZero
;;;;;;;;;;;;;;;;;;;;;;
; Copies
; Params:
;   rsi <= the src
;
; Nidified registers:
;   rax
;;;;;;;;;;;;;;;;;;;;;;;
Fr_rawZero:
        xor     rax, rax

        mov     [rdi + 0], rax

        ret

;;;;;;;;;;;;;;;;;;;;;;
; rawSwap
;;;;;;;;;;;;;;;;;;;;;;
; Copies
; Params:
;   rdi <= a
;   rsi <= p
;
; Nidified registers:
;   rax
;;;;;;;;;;;;;;;;;;;;;;;
Fr_rawSwap:

        mov     rax, [rsi + 0]
        mov     rcx, [rdi + 0]
        mov     [rdi + 0], rax
        mov     [rsi + 0], rbx

        ret


;;;;;;;;;;;;;;;;;;;;;;
; copy an array of integers
;;;;;;;;;;;;;;;;;;;;;;
; Copies
; Params:
;   rsi <= the src
;   rdi <= the dest
;   rdx <= number of integers to copy
;
; Nidified registers:
;   rax
;;;;;;;;;;;;;;;;;;;;;;;
Fr_copyn:
Fr_copyn_loop:
        mov     r8, rsi
        mov     r9, rdi
        mov     rax, 2
        mul     rdx
        mov     rcx, rax
        cld
   rep  movsq
        mov     rsi, r8
        mov     rdi, r9
        ret

;;;;;;;;;;;;;;;;;;;;;;
; rawCopyS2L
;;;;;;;;;;;;;;;;;;;;;;
; Convert a 64 bit integer to a long format field element
; Params:
;   rsi <= the integer
;   rdi <= Pointer to the overwritted element
;
; Nidified registers:
;   rax
;;;;;;;;;;;;;;;;;;;;;;;

rawCopyS2L:
        mov     al, 0x80
        shl     rax, 56
        mov     [rdi], rax    ; set the result to LONG normal

        cmp     rsi, 0
        js      u64toLong_adjust_neg

        mov     [rdi + 8], rsi
        xor     rax, rax

        ret

u64toLong_adjust_neg:
        add    rsi, [q]         ; Set the first digit
        mov    [rdi + 8], rsi   ;

        mov    rsi, -1          ; all ones

        ret

;;;;;;;;;;;;;;;;;;;;;;
; toInt
;;;;;;;;;;;;;;;;;;;;;;
; Convert a 64 bit integer to a long format field element
; Params:
;   rsi <= Pointer to the element
; Returs:
;   rax <= The value
;;;;;;;;;;;;;;;;;;;;;;;
Fr_toInt:
        mov     rax, [rdi]
        bt      rax, 63
        jc      Fr_long
        movsx   rax, eax
        ret

Fr_long:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        bt      rax, 62
        jnc     Fr_longNormal
Fr_longMontgomery:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


Fr_longNormal:
        mov     rax, [rdi + 8]
        mov     rcx, rax
        shr     rcx, 31
        jnz     Fr_longNeg

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

Fr_longNeg:
        mov     rax, [rdi + 8]
        sub     rax, [q]
        jnc     Fr_longErr

        mov     rcx, rax
        sar     rcx, 31
        add     rcx, 1
        jnz     Fr_longErr
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

Fr_longErr:
        push    rdi
        mov     rdi, 0
        call    Fr_fail
        pop     rdi
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret





Fr_rawMMul:
    push r13
    push r12
    mov rcx,rdx
    mov r9,[ np ]
    xor r10,r10

; FirstLoop
    mov rdx,[rsi + 0]
    mulx rax,r11,[rcx]
    mov r12,r10
    adcx r12,rax
    mov r13,r10
    adcx r13,r10
; SecondLoop
    mov rdx,r9
    mulx rax,rdx,r11
    mulx r8,rax,[q]
    adcx rax,r11
    mov r11,r10
    adcx r11,r8
    adox r11,r12
    mov r12,r10
    adcx r12,r10
    adox r12,r13

;comparison
    test r12,r12
jnz Fr_rawMMul_sq
    cmp r11,[q + 0]
    jc Fr_rawMMul_done
    jnz Fr_rawMMul_sq
Fr_rawMMul_sq:
    sub r11,[q +0]
Fr_rawMMul_done:
    mov [rdi + 0],r11
    pop r12
    pop r13
    ret
Fr_rawMSquare:
    push r13
    push r12
    mov rcx,rdx
    mov r9,[ np ]
    xor r10,r10

; FirstLoop
    mov rdx,[rsi + 0]
    mulx rax,r11,rdx
    mov r12,r10
    adcx r12,rax
    mov r13,r10
    adcx r13,r10
; SecondLoop
    mov rdx,r9
    mulx rax,rdx,r11
    mulx r8,rax,[q]
    adcx rax,r11
    mov r11,r10
    adcx r11,r8
    adox r11,r12
    mov r12,r10
    adcx r12,r10
    adox r12,r13

;comparison
    test r12,r12
jnz Fr_rawMSquare_sq
    cmp r11,[q + 0]
    jc Fr_rawMSquare_done
    jnz Fr_rawMSquare_sq
Fr_rawMSquare_sq:
    sub r11,[q +0]
Fr_rawMSquare_done:
    mov [rdi + 0],r11
    pop r12
    pop r13
    ret
Fr_rawMMul1:
    push r13
    push r12
    mov rcx,rdx
    mov r9,[ np ]
    xor r10,r10

; FirstLoop
    mov rdx,rcx
    mulx rax,r11,[rsi]
    mov r12,r10
    adcx r12,rax
    mov r13,r10
    adcx r13,r10
; SecondLoop
    mov rdx,r9
    mulx rax,rdx,r11
    mulx r8,rax,[q]
    adcx rax,r11
    mov r11,r10
    adcx r11,r8
    adox r11,r12
    mov r12,r10
    adcx r12,r10
    adox r12,r13

;comparison
    test r12,r12
jnz Fr_rawMMul1_sq
    cmp r11,[q + 0]
    jc Fr_rawMMul1_done
    jnz Fr_rawMMul1_sq
Fr_rawMMul1_sq:
    sub r11,[q +0]
Fr_rawMMul1_done:
    mov [rdi + 0],r11
    pop r12
    pop r13
    ret
Fr_rawFromMontgomery:
    push r13
    push r12
    mov rcx,rdx
    mov r9,[ np ]
    xor r10,r10

; FirstLoop
    mov r11,[rsi +0]
    mov r12,r10
    mov r13,r10
; SecondLoop
    mov rdx,r9
    mulx rax,rdx,r11
    mulx r8,rax,[q]
    adcx rax,r11
    mov r11,r10
    adcx r11,r8
    adox r11,r12
    mov r12,r10
    adcx r12,r10
    adox r12,r13

;comparison
    test r12,r12
jnz Fr_rawFromMontgomery_sq
    cmp r11,[q + 0]
    jc Fr_rawFromMontgomery_done
    jnz Fr_rawFromMontgomery_sq
Fr_rawFromMontgomery_sq:
    sub r11,[q +0]
Fr_rawFromMontgomery_done:
    mov [rdi + 0],r11
    pop r12
    pop r13
    ret

;;;;;;;;;;;;;;;;;;;;;;
; rawToMontgomery
;;;;;;;;;;;;;;;;;;;;;;
; Convert a number to Montgomery
;   rdi <= Pointer destination element
;   rsi <= Pointer to src element
;;;;;;;;;;;;;;;;;;;;
Fr_rawToMontgomery:
    push    rdx
    lea     rdx, [R2]
    call    Fr_rawMMul
    pop     rdx
    ret

;;;;;;;;;;;;;;;;;;;;;;
; toMontgomery
;;;;;;;;;;;;;;;;;;;;;;
; Convert a number to Montgomery
;   rdi <= Destination
;   rdi <= Pointer element to convert
; Modified registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;
Fr_toMontgomery:
    mov     rax, [rsi]
    bt      rax, 62                     ; check if montgomery
    jc      toMontgomery_doNothing
    bt      rax, 63
    jc      toMontgomeryLong

toMontgomeryShort:
    movsx   rdx, eax
    mov     [rdi], rdx
    add     rdi, 8
    lea     rsi, [R2]
    cmp     rdx, 0
    js      negMontgomeryShort
posMontgomeryShort:
    call    Fr_rawMMul1
    sub     rdi, 8
            mov r11b, 0x40
        shl r11d, 24
        mov [rdi+4], r11d
    ret

negMontgomeryShort:
    neg     rdx              ; Do the multiplication positive and then negate the result.
    call    Fr_rawMMul1
    mov     rsi, rdi
    call    rawNegL
    sub     rdi, 8
            mov r11b, 0x40
        shl r11d, 24
        mov [rdi+4], r11d
    ret


toMontgomeryLong:
    mov     [rdi], rax
    add     rdi, 8
    add     rsi, 8
    lea     rdx, [R2]
    call    Fr_rawMMul
    sub     rsi, 8
    sub     rdi, 8
            mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d
    ret


toMontgomery_doNothing:
    call   Fr_copy
    ret

;;;;;;;;;;;;;;;;;;;;;;
; toNormal
;;;;;;;;;;;;;;;;;;;;;;
; Convert a number from Montgomery
;   rdi <= Destination
;   rsi <= Pointer element to convert
; Modified registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;
Fr_toNormal:
    mov     rax, [rsi]
    bt      rax, 62                     ; check if montgomery
    jnc     toNormal_doNothing
    bt      rax, 63                     ; if short, it means it's converted
    jnc     toNormal_doNothing

toNormalLong:
    add     rdi, 8
    add     rsi, 8
    call    Fr_rawFromMontgomery
    sub     rsi, 8
    sub     rdi, 8
            mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d
    ret

toNormal_doNothing:
    call   Fr_copy
    ret

;;;;;;;;;;;;;;;;;;;;;;
; toLongNormal
;;;;;;;;;;;;;;;;;;;;;;
; Convert a number to long normal
;   rdi <= Destination
;   rsi <= Pointer element to convert
; Modified registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;
Fr_toLongNormal:
    mov     rax, [rsi]
    bt      rax, 63                     ; check if long
    jnc     toLongNormal_fromShort
    bt      rax, 62                     ; check if montgomery
    jc      toLongNormal_fromMontgomery
    call    Fr_copy              ; It is already long
    ret

toLongNormal_fromMontgomery:
    add     rdi, 8
    add     rsi, 8
    call    Fr_rawFromMontgomery
    sub     rsi, 8
    sub     rdi, 8
            mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d
    ret

toLongNormal_fromShort:
    mov     r8, rsi                     ; save rsi
    movsx   rsi, eax
    call    rawCopyS2L
    mov     rsi, r8                     ; recover rsi
            mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d
    ret












;;;;;;;;;;;;;;;;;;;;;;
; add
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_add:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        mov    rax, [rsi]
        mov    rcx, [rdx]
        bt     rax, 63          ; Check if is short first operand
        jc     add_l1
        bt     rcx, 63          ; Check if is short second operand
        jc     add_s1l2

add_s1s2:                       ; Both operands are short

        xor    rdx, rdx
        mov    edx, eax
        add    edx, ecx
        jo     add_manageOverflow   ; rsi already is the 64bits result

        mov    [rdi], rdx       ; not necessary to adjust so just save and return
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

add_manageOverflow:                 ; Do the operation in 64 bits
        push   rsi
        movsx  rsi, eax
        movsx  rdx, ecx
        add    rsi, rdx
        call   rawCopyS2L
        pop    rsi
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

add_l1:
        bt     rcx, 63          ; Check if is short second operand
        jc     add_l1l2

;;;;;;;;
add_l1s2:
        bt     rax, 62          ; check if montgomery first
        jc     add_l1ms2
add_l1ns2:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        add rsi, 8
        movsx rdx, ecx
        add rdi, 8
        cmp rdx, 0
        
        jns tmp_1
        neg rdx
        call rawSubLS
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret
tmp_1:
        call rawAddLS
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret



add_l1ms2:
        bt     rcx, 62          ; check if montgomery second
        jc     add_l1ms2m
add_l1ms2n:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toMontgomery
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawAddLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


add_l1ms2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawAddLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret



;;;;;;;;
add_s1l2:
        bt     rcx, 62          ; check if montgomery second
        jc     add_s1l2m
add_s1l2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        lea rsi, [rdx + 8]
        movsx rdx, eax
        add rdi, 8
        cmp rdx, 0
        
        jns tmp_2
        neg rdx
        call rawSubLS
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret
tmp_2:
        call rawAddLS
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


add_s1l2m:
        bt     rax, 62          ; check if montgomery first
        jc     add_s1ml2m
add_s1nl2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toMontgomery
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawAddLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


add_s1ml2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawAddLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


;;;;
add_l1l2:
        bt     rax, 62          ; check if montgomery first
        jc     add_l1ml2
add_l1nl2:
        bt     rcx, 62          ; check if montgomery second
        jc     add_l1nl2m
add_l1nl2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawAddLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


add_l1nl2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toMontgomery
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawAddLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


add_l1ml2:
        bt     rcx, 62          ; check if montgomery seconf
        jc     add_l1ml2m
add_l1ml2n:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toMontgomery
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawAddLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


add_l1ml2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawAddLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




;;;;;;;;;;;;;;;;;;;;;;
; rawAddLL
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of type long
; Params:
;   rsi <= Pointer to the long data of element 1
;   rdx <= Pointer to the long data of element 2
;   rdi <= Pointer to the long data of result
; Modified Registers:
;    rax
;;;;;;;;;;;;;;;;;;;;;;
rawAddLL:
Fr_rawAdd:
        ; Add component by component with carry

        mov rax, [rsi + 0]
        add rax, [rdx + 0]
        mov [rdi + 0], rax

        jc rawAddLL_sq   ; if overflow, substract q

        ; Compare with q


        cmp rax, [q + 0]
        jc rawAddLL_done        ; q is bigget so done.
        jnz rawAddLL_sq         ; q is lower

        ; If equal substract q
rawAddLL_sq:

        mov rax, [q + 0]
        sub [rdi + 0], rax

rawAddLL_done:
        ret


;;;;;;;;;;;;;;;;;;;;;;
; rawAddLS
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of type long
; Params:
;   rdi <= Pointer to the long data of result
;   rsi <= Pointer to the long data of element 1
;   rdx <= Value to be added
;;;;;;;;;;;;;;;;;;;;;;
rawAddLS:
        ; Add component by component with carry

        add rdx, [rsi]
        mov [rdi] ,rdx

        jc rawAddLS_sq   ; if overflow, substract q

        ; Compare with q

        mov rax, [rdi + 0]
        cmp rax, [q + 0]
        jc rawAddLS_done        ; q is bigget so done.
        jnz rawAddLS_sq         ; q is lower

        ; If equal substract q
rawAddLS_sq:

        mov rax, [q + 0]
        sub [rdi + 0], rax

rawAddLS_done:
        ret















;;;;;;;;;;;;;;;;;;;;;;
; sub
;;;;;;;;;;;;;;;;;;;;;;
; Substracts two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_sub:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        mov    rax, [rsi]
        mov    rcx, [rdx]
        bt     rax, 63          ; Check if is long first operand
        jc     sub_l1
        bt     rcx, 63          ; Check if is long second operand
        jc     sub_s1l2

sub_s1s2:                       ; Both operands are short

        xor    rdx, rdx
        mov    edx, eax
        sub    edx, ecx
        jo     sub_manageOverflow   ; rsi already is the 64bits result

        mov    [rdi], rdx       ; not necessary to adjust so just save and return
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

sub_manageOverflow:                 ; Do the operation in 64 bits
        push   rsi
        movsx  rsi, eax
        movsx  rdx, ecx
        sub    rsi, rdx
        call   rawCopyS2L
        pop    rsi
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

sub_l1:
        bt     rcx, 63          ; Check if is short second operand
        jc     sub_l1l2

;;;;;;;;
sub_l1s2:
        bt     rax, 62          ; check if montgomery first
        jc     sub_l1ms2
sub_l1ns2:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        add rsi, 8
        movsx rdx, ecx
        add rdi, 8
        cmp rdx, 0
        
        jns tmp_3
        neg rdx
        call rawAddLS
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret
tmp_3:
        call rawSubLS
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


sub_l1ms2:
        bt     rcx, 62          ; check if montgomery second
        jc     sub_l1ms2m
sub_l1ms2n:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toMontgomery
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawSubLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


sub_l1ms2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawSubLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret



;;;;;;;;
sub_s1l2:
        bt     rcx, 62          ; check if montgomery first
        jc     sub_s1l2m
sub_s1l2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        cmp eax, 0
        
        js tmp_4

        ; First Operand is positive
        push rsi
        add rdi, 8
        movsx rsi, eax
        add rdx, 8
        call rawSubSL
        sub rdi, 8
        pop rsi
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_4:   ; First operand is negative
        push rsi
        lea rsi, [rdx + 8]
        movsx rdx, eax
        add rdi, 8
        neg rdx
        call rawNegLS
        sub rdi, 8
        pop rsi
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


sub_s1l2m:
        bt     rax, 62          ; check if montgomery second
        jc     sub_s1ml2m
sub_s1nl2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toMontgomery
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawSubLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


sub_s1ml2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawSubLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


;;;;
sub_l1l2:
        bt     rax, 62          ; check if montgomery first
        jc     sub_l1ml2
sub_l1nl2:
        bt     rcx, 62          ; check if montgomery second
        jc     sub_l1nl2m
sub_l1nl2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawSubLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


sub_l1nl2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toMontgomery
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawSubLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


sub_l1ml2:
        bt     rcx, 62          ; check if montgomery seconf
        jc     sub_l1ml2m
sub_l1ml2n:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toMontgomery
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawSubLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


sub_l1ml2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call rawSubLL
        sub rdi, 8
        sub rsi, 8
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret



;;;;;;;;;;;;;;;;;;;;;;
; rawSubLS
;;;;;;;;;;;;;;;;;;;;;;
; Substracts a short element from the long element
; Params:
;   rdi <= Pointer to the long data of result
;   rsi <= Pointer to the long data of element 1 where will be substracted
;   rdx <= Value to be substracted
;   [rdi] = [rsi] - rdx
; Modified Registers:
;    rax
;;;;;;;;;;;;;;;;;;;;;;
rawSubLS:
        ; Substract first digit

        mov rax, [rsi]
        sub rax, rdx
        mov [rdi] ,rax
        mov rdx, 0

        jnc rawSubLS_done   ; if overflow, add q

        ; Add q
rawSubLS_aq:

        mov rax, [q + 0]
        add [rdi + 0], rax

rawSubLS_done:
        ret


;;;;;;;;;;;;;;;;;;;;;;
; rawSubSL
;;;;;;;;;;;;;;;;;;;;;;
; Substracts a long element from a short element
; Params:
;   rdi <= Pointer to the long data of result
;   rsi <= Value from where will bo substracted
;   rdx <= Pointer to long of the value to be substracted
;
;   [rdi] = rsi - [rdx]
; Modified Registers:
;    rax
;;;;;;;;;;;;;;;;;;;;;;
rawSubSL:
        ; Substract first digit
        sub rsi, [rdx]
        mov [rdi] ,rsi


        jnc rawSubSL_done   ; if overflow, add q

        ; Add q
rawSubSL_aq:

        mov rax, [q + 0]
        add [rdi + 0], rax

rawSubSL_done:
        ret

;;;;;;;;;;;;;;;;;;;;;;
; rawSubLL
;;;;;;;;;;;;;;;;;;;;;;
; Substracts a long element from a short element
; Params:
;   rdi <= Pointer to the long data of result
;   rsi <= Pointer to long from where substracted
;   rdx <= Pointer to long of the value to be substracted
;
;   [rdi] = [rsi] - [rdx]
; Modified Registers:
;    rax
;;;;;;;;;;;;;;;;;;;;;;
rawSubLL:
Fr_rawSub:
        ; Substract first digit

        mov rax, [rsi + 0]
        sub rax, [rdx + 0]
        mov [rdi + 0], rax

        jnc rawSubLL_done   ; if overflow, add q

        ; Add q
rawSubLL_aq:

        mov rax, [q + 0]
        add [rdi + 0], rax

rawSubLL_done:
        ret

;;;;;;;;;;;;;;;;;;;;;;
; rawNegLS
;;;;;;;;;;;;;;;;;;;;;;
; Substracts a long element and a short element form 0
; Params:
;   rdi <= Pointer to the long data of result
;   rsi <= Pointer to long from where substracted
;   rdx <= short value to be substracted too
;
;   [rdi] = -[rsi] - rdx
; Modified Registers:
;    rax
;;;;;;;;;;;;;;;;;;;;;;
rawNegLS:
        mov rax, [q]
        sub rax, rdx
        mov [rdi], rax

        setc dl


        mov rax, [rdi + 0 ]
        sub rax, [rsi + 0]
        mov [rdi + 0], rax


        setc dh
        or dl, dh
        jz rawNegSL_done

        ; it is a negative value, so add q

        mov rax, [q + 0]
        add [rdi + 0], rax


rawNegSL_done:
        ret







;;;;;;;;;;;;;;;;;;;;;;
; neg
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element to be negated
;   rdi <= Pointer to result
;   [rdi] = -[rsi]
;;;;;;;;;;;;;;;;;;;;;;
Fr_neg:
        mov    rax, [rsi]
        bt     rax, 63          ; Check if is short first operand
        jc     neg_l

neg_s:                          ; Operand is short

        neg    eax
        jo     neg_manageOverflow   ; Check if overflow. (0x80000000 is the only case)

        mov    [rdi], rax           ; not necessary to adjust so just save and return
        ret

neg_manageOverflow:                 ; Do the operation in 64 bits
        push   rsi
        movsx  rsi, eax
        neg    rsi
        call   rawCopyS2L
        pop    rsi
        ret



neg_l:
        mov [rdi], rax          ; Copy the type

        add rdi, 8
        add rsi, 8
        call rawNegL
        sub rdi, 8
        sub rsi, 8
        ret



;;;;;;;;;;;;;;;;;;;;;;
; rawNeg
;;;;;;;;;;;;;;;;;;;;;;
; Negates a value
; Params:
;   rdi <= Pointer to the long data of result
;   rsi <= Pointer to the long data of element 1
;
;   [rdi] = - [rsi]
;;;;;;;;;;;;;;;;;;;;;;
rawNegL:
Fr_rawNeg:
        ; Compare is zero

        xor rax, rax

        cmp [rsi + 0], rax
        jnz doNegate

        ; it's zero so just set to zero

        mov [rdi + 0], rax

        ret
doNegate:

        mov rax, [q + 0]
        sub rax, [rsi + 0]
        mov [rdi + 0], rax

        ret



















;;;;;;;;;;;;;;;;;;;;;;
; square
;;;;;;;;;;;;;;;;;;;;;;
; Squares a field element
; Params:
;   rsi <= Pointer to element 1
;   rdi <= Pointer to result
;   [rdi] = [rsi] * [rsi]
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_square:
        mov    r8, [rsi]
        bt     r8, 63          ; Check if is short first operand
        jc     square_l1

square_s1:                       ; Both operands are short

        xor    rax, rax
        mov    eax, r8d
        imul   eax
        jo     square_manageOverflow   ; rsi already is the 64bits result

        mov    [rdi], rax       ; not necessary to adjust so just save and return

square_manageOverflow:                 ; Do the operation in 64 bits
        push   rsi
        movsx  rax, r8d
        imul   rax
        mov    rsi, rax
        call   rawCopyS2L
        pop    rsi

        ret

square_l1:
        bt     r8, 62          ; check if montgomery first
        jc     square_l1m
square_l1n:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        call Fr_rawMSquare
        sub rdi, 8
        sub rsi, 8


        push rsi
        add rdi, 8
        mov rsi, rdi
        lea rdx, [R3]
        call Fr_rawMMul
        sub rdi, 8
        pop rsi

        ret

square_l1m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        call Fr_rawMSquare
        sub rdi, 8
        sub rsi, 8

        ret



;;;;;;;;;;;;;;;;;;;;;;
; mul
;;;;;;;;;;;;;;;;;;;;;;
; Multiplies two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result
;   [rdi] = [rsi] * [rdi]
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_mul:
        mov    r8, [rsi]
        mov    r9, [rdx]
        bt     r8, 63          ; Check if is short first operand
        jc     mul_l1
        bt     r9, 63          ; Check if is short second operand
        jc     mul_s1l2

mul_s1s2:                       ; Both operands are short

        xor    rax, rax
        mov    eax, r8d
        imul   r9d
        jo     mul_manageOverflow   ; rsi already is the 64bits result

        mov    [rdi], rax       ; not necessary to adjust so just save and return

mul_manageOverflow:                 ; Do the operation in 64 bits
        push   rsi
        movsx  rax, r8d
        movsx  rcx, r9d
        imul   rcx
        mov    rsi, rax
        call   rawCopyS2L
        pop    rsi

        ret

mul_l1:
        bt     r9, 63          ; Check if is short second operand
        jc     mul_l1l2

;;;;;;;;
mul_l1s2:
        bt     r8, 62          ; check if montgomery first
        jc     mul_l1ms2
mul_l1ns2:
        bt     r9, 62          ; check if montgomery first
        jc     mul_l1ns2m
mul_l1ns2n:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        push rsi
        add rsi, 8
        movsx rdx, r9d
        add rdi, 8
        cmp rdx, 0
        
        jns tmp_5
        neg rdx
        call Fr_rawMMul1
        mov rsi, rdi
        call rawNegL
        sub rdi, 8
        pop rsi
        
        jmp tmp_6
tmp_5:
        call Fr_rawMMul1
        sub rdi, 8
        pop rsi
tmp_6:



        push rsi
        add rdi, 8
        mov rsi, rdi
        lea rdx, [R3]
        call Fr_rawMMul
        sub rdi, 8
        pop rsi

        ret


mul_l1ns2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call Fr_rawMMul
        sub rdi, 8
        sub rsi, 8

        ret


mul_l1ms2:
        bt     r9, 62          ; check if montgomery second
        jc     mul_l1ms2m
mul_l1ms2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        push rsi
        add rsi, 8
        movsx rdx, r9d
        add rdi, 8
        cmp rdx, 0
        
        jns tmp_7
        neg rdx
        call Fr_rawMMul1
        mov rsi, rdi
        call rawNegL
        sub rdi, 8
        pop rsi
        
        jmp tmp_8
tmp_7:
        call Fr_rawMMul1
        sub rdi, 8
        pop rsi
tmp_8:


        ret

mul_l1ms2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call Fr_rawMMul
        sub rdi, 8
        sub rsi, 8

        ret


;;;;;;;;
mul_s1l2:
        bt     r8, 62          ; check if montgomery first
        jc     mul_s1ml2
mul_s1nl2:
        bt     r9, 62          ; check if montgomery first
        jc     mul_s1nl2m
mul_s1nl2n:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        push rsi
        lea rsi, [rdx + 8]
        movsx rdx, r8d
        add rdi, 8
        cmp rdx, 0
        
        jns tmp_9
        neg rdx
        call Fr_rawMMul1
        mov rsi, rdi
        call rawNegL
        sub rdi, 8
        pop rsi
        
        jmp tmp_10
tmp_9:
        call Fr_rawMMul1
        sub rdi, 8
        pop rsi
tmp_10:



        push rsi
        add rdi, 8
        mov rsi, rdi
        lea rdx, [R3]
        call Fr_rawMMul
        sub rdi, 8
        pop rsi

        ret

mul_s1nl2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        push rsi
        lea rsi, [rdx + 8]
        movsx rdx, r8d
        add rdi, 8
        cmp rdx, 0
        
        jns tmp_11
        neg rdx
        call Fr_rawMMul1
        mov rsi, rdi
        call rawNegL
        sub rdi, 8
        pop rsi
        
        jmp tmp_12
tmp_11:
        call Fr_rawMMul1
        sub rdi, 8
        pop rsi
tmp_12:


        ret

mul_s1ml2:
        bt     r9, 62          ; check if montgomery first
        jc     mul_s1ml2m
mul_s1ml2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call Fr_rawMMul
        sub rdi, 8
        sub rsi, 8

        ret

mul_s1ml2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call Fr_rawMMul
        sub rdi, 8
        sub rsi, 8

        ret

;;;;
mul_l1l2:
        bt     r8, 62          ; check if montgomery first
        jc     mul_l1ml2
mul_l1nl2:
        bt     r9, 62          ; check if montgomery second
        jc     mul_l1nl2m
mul_l1nl2n:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call Fr_rawMMul
        sub rdi, 8
        sub rsi, 8


        push rsi
        add rdi, 8
        mov rsi, rdi
        lea rdx, [R3]
        call Fr_rawMMul
        sub rdi, 8
        pop rsi

        ret

mul_l1nl2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call Fr_rawMMul
        sub rdi, 8
        sub rsi, 8

        ret

mul_l1ml2:
        bt     r9, 62          ; check if montgomery seconf
        jc     mul_l1ml2m
mul_l1ml2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call Fr_rawMMul
        sub rdi, 8
        sub rsi, 8

        ret

mul_l1ml2m:
        mov r11b, 0xC0
        shl r11d, 24
        mov [rdi+4], r11d

        add rdi, 8
        add rsi, 8
        add rdx, 8
        call Fr_rawMMul
        sub rdi, 8
        sub rsi, 8

        ret



















;;;;;;;;;;;;;;;;;;;;;;
; band
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_band:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        mov    rax, [rsi]
        mov    rcx, [rdx]
        bt     rax, 63          ; Check if is short first operand
        jc     and_l1
        bt     rcx, 63          ; Check if is short second operand
        jc     and_s1l2

and_s1s2:

        cmp    eax, 0
        
        js     tmp_13

        cmp    ecx, 0
        js     tmp_13
        xor    rdx, rdx         ; both ops are positive so do the op and return
        mov    edx, eax
        and    edx, ecx
        mov    [rdi], rdx       ; not necessary to adjust so just save and return
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_13:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx



        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_15        ; q is bigget so done.
        jnz tmp_14         ; q is lower

        ; If equal substract q
tmp_14:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_15:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret






and_l1:
        bt     rcx, 63          ; Check if is short second operand
        jc     and_l1l2


and_l1s2:
        bt     rax, 62          ; check if montgomery first
        jc     and_l1ms2
and_l1ns2:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov    rcx, [rdx]
        cmp    ecx, 0
        
        js     tmp_16
        movsx  rax, ecx
        and rax, [rsi +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_18        ; q is bigget so done.
        jnz tmp_17         ; q is lower

        ; If equal substract q
tmp_17:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_18:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_16:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_20        ; q is bigget so done.
        jnz tmp_19         ; q is lower

        ; If equal substract q
tmp_19:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_20:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




and_l1ms2:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov    rcx, [rdx]
        cmp    ecx, 0
        
        js     tmp_21
        movsx  rax, ecx
        and rax, [rsi +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_23        ; q is bigget so done.
        jnz tmp_22         ; q is lower

        ; If equal substract q
tmp_22:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_23:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_21:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_25        ; q is bigget so done.
        jnz tmp_24         ; q is lower

        ; If equal substract q
tmp_24:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_25:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret





and_s1l2:
        bt     rcx, 62          ; check if montgomery first
        jc     and_s1l2m
and_s1l2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov    eax, [rsi]
        cmp    eax, 0
        
        js     tmp_26
        and rax, [rdx +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_28        ; q is bigget so done.
        jnz tmp_27         ; q is lower

        ; If equal substract q
tmp_27:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_28:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_26:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_30        ; q is bigget so done.
        jnz tmp_29         ; q is lower

        ; If equal substract q
tmp_29:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_30:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




and_s1l2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        mov    eax, [rsi]
        cmp    eax, 0
        
        js     tmp_31
        and rax, [rdx +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_33        ; q is bigget so done.
        jnz tmp_32         ; q is lower

        ; If equal substract q
tmp_32:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_33:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_31:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_35        ; q is bigget so done.
        jnz tmp_34         ; q is lower

        ; If equal substract q
tmp_34:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_35:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret





and_l1l2:
        bt     rax, 62          ; check if montgomery first
        jc     and_l1ml2
        bt     rcx, 62          ; check if montgomery first
        jc     and_l1nl2m
and_l1nl2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_37        ; q is bigget so done.
        jnz tmp_36         ; q is lower

        ; If equal substract q
tmp_36:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_37:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


and_l1nl2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi



        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_39        ; q is bigget so done.
        jnz tmp_38         ; q is lower

        ; If equal substract q
tmp_38:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_39:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


and_l1ml2:
        bt     rcx, 62          ; check if montgomery first
        jc     and_l1ml2m
and_l1ml2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx



        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_41        ; q is bigget so done.
        jnz tmp_40         ; q is lower

        ; If equal substract q
tmp_40:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_41:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


and_l1ml2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi



        mov rax, [rsi + 8]
        and rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_43        ; q is bigget so done.
        jnz tmp_42         ; q is lower

        ; If equal substract q
tmp_42:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_43:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret



;;;;;;;;;;;;;;;;;;;;;;
; bor
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_bor:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        mov    rax, [rsi]
        mov    rcx, [rdx]
        bt     rax, 63          ; Check if is short first operand
        jc     or_l1
        bt     rcx, 63          ; Check if is short second operand
        jc     or_s1l2

or_s1s2:

        cmp    eax, 0
        
        js     tmp_44

        cmp    ecx, 0
        js     tmp_44
        xor    rdx, rdx         ; both ops are positive so do the op and return
        mov    edx, eax
        or    edx, ecx
        mov    [rdi], rdx       ; not necessary to adjust so just save and return
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_44:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx



        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_46        ; q is bigget so done.
        jnz tmp_45         ; q is lower

        ; If equal substract q
tmp_45:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_46:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret






or_l1:
        bt     rcx, 63          ; Check if is short second operand
        jc     or_l1l2


or_l1s2:
        bt     rax, 62          ; check if montgomery first
        jc     or_l1ms2
or_l1ns2:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov    rcx, [rdx]
        cmp    ecx, 0
        
        js     tmp_47
        movsx  rax, ecx
        or rax, [rsi +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_49        ; q is bigget so done.
        jnz tmp_48         ; q is lower

        ; If equal substract q
tmp_48:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_49:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_47:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_51        ; q is bigget so done.
        jnz tmp_50         ; q is lower

        ; If equal substract q
tmp_50:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_51:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




or_l1ms2:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov    rcx, [rdx]
        cmp    ecx, 0
        
        js     tmp_52
        movsx  rax, ecx
        or rax, [rsi +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_54        ; q is bigget so done.
        jnz tmp_53         ; q is lower

        ; If equal substract q
tmp_53:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_54:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_52:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_56        ; q is bigget so done.
        jnz tmp_55         ; q is lower

        ; If equal substract q
tmp_55:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_56:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret





or_s1l2:
        bt     rcx, 62          ; check if montgomery first
        jc     or_s1l2m
or_s1l2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov    eax, [rsi]
        cmp    eax, 0
        
        js     tmp_57
        or rax, [rdx +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_59        ; q is bigget so done.
        jnz tmp_58         ; q is lower

        ; If equal substract q
tmp_58:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_59:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_57:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_61        ; q is bigget so done.
        jnz tmp_60         ; q is lower

        ; If equal substract q
tmp_60:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_61:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




or_s1l2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        mov    eax, [rsi]
        cmp    eax, 0
        
        js     tmp_62
        or rax, [rdx +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_64        ; q is bigget so done.
        jnz tmp_63         ; q is lower

        ; If equal substract q
tmp_63:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_64:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_62:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_66        ; q is bigget so done.
        jnz tmp_65         ; q is lower

        ; If equal substract q
tmp_65:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_66:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret





or_l1l2:
        bt     rax, 62          ; check if montgomery first
        jc     or_l1ml2
        bt     rcx, 62          ; check if montgomery first
        jc     or_l1nl2m
or_l1nl2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_68        ; q is bigget so done.
        jnz tmp_67         ; q is lower

        ; If equal substract q
tmp_67:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_68:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


or_l1nl2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi



        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_70        ; q is bigget so done.
        jnz tmp_69         ; q is lower

        ; If equal substract q
tmp_69:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_70:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


or_l1ml2:
        bt     rcx, 62          ; check if montgomery first
        jc     or_l1ml2m
or_l1ml2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx



        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_72        ; q is bigget so done.
        jnz tmp_71         ; q is lower

        ; If equal substract q
tmp_71:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_72:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


or_l1ml2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi



        mov rax, [rsi + 8]
        or rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_74        ; q is bigget so done.
        jnz tmp_73         ; q is lower

        ; If equal substract q
tmp_73:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_74:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret



;;;;;;;;;;;;;;;;;;;;;;
; bxor
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_bxor:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        mov    rax, [rsi]
        mov    rcx, [rdx]
        bt     rax, 63          ; Check if is short first operand
        jc     xor_l1
        bt     rcx, 63          ; Check if is short second operand
        jc     xor_s1l2

xor_s1s2:

        cmp    eax, 0
        
        js     tmp_75

        cmp    ecx, 0
        js     tmp_75
        xor    rdx, rdx         ; both ops are positive so do the op and return
        mov    edx, eax
        xor    edx, ecx
        mov    [rdi], rdx       ; not necessary to adjust so just save and return
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_75:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx



        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_77        ; q is bigget so done.
        jnz tmp_76         ; q is lower

        ; If equal substract q
tmp_76:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_77:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret






xor_l1:
        bt     rcx, 63          ; Check if is short second operand
        jc     xor_l1l2


xor_l1s2:
        bt     rax, 62          ; check if montgomery first
        jc     xor_l1ms2
xor_l1ns2:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov    rcx, [rdx]
        cmp    ecx, 0
        
        js     tmp_78
        movsx  rax, ecx
        xor rax, [rsi +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_80        ; q is bigget so done.
        jnz tmp_79         ; q is lower

        ; If equal substract q
tmp_79:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_80:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_78:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_82        ; q is bigget so done.
        jnz tmp_81         ; q is lower

        ; If equal substract q
tmp_81:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_82:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




xor_l1ms2:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov    rcx, [rdx]
        cmp    ecx, 0
        
        js     tmp_83
        movsx  rax, ecx
        xor rax, [rsi +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_85        ; q is bigget so done.
        jnz tmp_84         ; q is lower

        ; If equal substract q
tmp_84:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_85:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_83:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_87        ; q is bigget so done.
        jnz tmp_86         ; q is lower

        ; If equal substract q
tmp_86:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_87:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret





xor_s1l2:
        bt     rcx, 62          ; check if montgomery first
        jc     xor_s1l2m
xor_s1l2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov    eax, [rsi]
        cmp    eax, 0
        
        js     tmp_88
        xor rax, [rdx +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_90        ; q is bigget so done.
        jnz tmp_89         ; q is lower

        ; If equal substract q
tmp_89:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_90:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_88:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_92        ; q is bigget so done.
        jnz tmp_91         ; q is lower

        ; If equal substract q
tmp_91:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_92:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




xor_s1l2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        mov    eax, [rsi]
        cmp    eax, 0
        
        js     tmp_93
        xor rax, [rdx +8]
        mov    [rdi+8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_95        ; q is bigget so done.
        jnz tmp_94         ; q is lower

        ; If equal substract q
tmp_94:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_95:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

tmp_93:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_97        ; q is bigget so done.
        jnz tmp_96         ; q is lower

        ; If equal substract q
tmp_96:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_97:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret





xor_l1l2:
        bt     rax, 62          ; check if montgomery first
        jc     xor_l1ml2
        bt     rcx, 62          ; check if montgomery first
        jc     xor_l1nl2m
xor_l1nl2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d


        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_99        ; q is bigget so done.
        jnz tmp_98         ; q is lower

        ; If equal substract q
tmp_98:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_99:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


xor_l1nl2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi



        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_101        ; q is bigget so done.
        jnz tmp_100         ; q is lower

        ; If equal substract q
tmp_100:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_101:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


xor_l1ml2:
        bt     rcx, 62          ; check if montgomery first
        jc     xor_l1ml2m
xor_l1ml2n:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx



        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_103        ; q is bigget so done.
        jnz tmp_102         ; q is lower

        ; If equal substract q
tmp_102:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_103:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


xor_l1ml2m:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi



        mov rax, [rsi + 8]
        xor rax, [rdx + 8]

        and    rax, [lboMask]

        mov    [rdi + 8 ], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_105        ; q is bigget so done.
        jnz tmp_104         ; q is lower

        ; If equal substract q
tmp_104:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_105:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




;;;;;;;;;;;;;;;;;;;;;;
; bnot
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_bnot:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
                mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d

        mov     rax, [rsi]
        bt      rax, 63          ; Check if is long operand
        jc      bnot_l1
bnot_s:
        
        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp     bnot_l1n

bnot_l1:
        bt     rax, 62          ; check if montgomery first
        jnc    bnot_l1n

bnot_l1m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


bnot_l1n:

        mov    rax, [rsi + 8]
        not    rax

        and    rax, [lboMask]

        mov    [rdi + 8], rax


        
        

        ; Compare with q

        mov rax, [rdi + 8]
        cmp rax, [q + 0]
        jc tmp_107        ; q is bigget so done.
        jnz tmp_106         ; q is lower

        ; If equal substract q
tmp_106:

        mov rax, [q + 0]
        sub [rdi + 8], rax

tmp_107:

        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret




;;;;;;;;;;;;;;;;;;;;;;
; rawShr
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= how much is shifted
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
rawShr:
        cmp rdx, 0
        je Fr_rawCopy

        cmp rdx, 64
        jae Fr_rawZero

rawShr_nz:
        mov r8, rdx
        shr r8,6
        mov rcx, rdx
        and rcx, 0x3F
        jz rawShr_aligned
        mov ch, 64
        sub ch, cl

        mov r9, 1
        rol cx, 8
        shl r9, cl
        rol cx, 8
        sub r9, 1
        mov r10, r9
        not r10


        cmp r8, 0
        jae rawShr_if2_0

        mov rax, [rsi + r8*8 + 0 ]
        shr rax, cl
        and rax, r9
        mov r11, [rsi + r8*8 + 8 ]
        rol cx, 8
        shl r11, cl
        rol cx, 8
        and r11, r10
        or rax, r11
        mov [rdi + 0], rax

        jmp rawShr_endif_0
rawShr_if2_0:
        jne rawShr_else_0

        mov rax, [rsi + r8*8 + 0 ]
        shr rax, cl
        and rax, r9
        mov [rdi + 0], rax

        jmp rawShr_endif_0
rawShr_else_0:
        xor  rax, rax
        mov [rdi + 0], rax
rawShr_endif_0:


        ret

rawShr_aligned:

        cmp r8, 0
        ja rawShr_if3_0
        mov rax, [rsi + r8*8 + 0 ]
        mov [rdi + 0], rax
        jmp rawShr_endif3_0
rawShr_if3_0:
        xor rax, rax
        mov [rdi + 0], rax
rawShr_endif3_0:

        ret


;;;;;;;;;;;;;;;;;;;;;;
; rawShl
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= how much is shifted
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
rawShl:
        cmp rdx, 0
        je Fr_rawCopy
        
        cmp rdx, 64
        jae Fr_rawZero

        mov r8, rdx
        shr r8,6
        mov rcx, rdx
        and rcx, 0x3F
        jz rawShl_aligned
        mov ch, 64
        sub ch, cl


        mov r10, 1
        shl r10, cl
        sub r10, 1
        mov r9, r10
        not r9

        mov rdx, rsi
        mov rax, r8
        shl rax, 3
        sub rdx, rax


        cmp r8, 0
        jae rawShl_if2_0

        mov rax, [rdx + 0 ]
        shl rax, cl
        and rax, r9
        mov r11, [rdx + -8 ]
        rol cx, 8
        shr r11, cl
        rol cx, 8
        and r11, r10
        or rax, r11

        and rax, [lboMask]

        
        mov [rdi + 0], rax

        jmp rawShl_endif_0
rawShl_if2_0:
        jne rawShl_else_0

        mov rax, [rdx + 0 ]
        shl rax, cl
        and rax, r9

        and rax, [lboMask]

        
        mov [rdi + 0], rax

        jmp rawShl_endif_0
rawShl_else_0:
        xor rax, rax
        mov [rdi + 0], rax
rawShl_endif_0:



        
        

        ; Compare with q

        mov rax, [rdi + 0]
        cmp rax, [q + 0]
        jc tmp_109        ; q is bigget so done.
        jnz tmp_108         ; q is lower

        ; If equal substract q
tmp_108:

        mov rax, [q + 0]
        sub [rdi + 0], rax

tmp_109:

        ret;

rawShl_aligned:
        mov rdx, rsi
        mov rax, r8
        shl rax, 3
        sub rdx, rax


        cmp r8, 0
        ja rawShl_if3_0
        mov rax, [rdx + 0 ]

        and rax, [lboMask]
        
        mov [rdi + 0], rax
        jmp rawShl_endif3_0
rawShl_if3_0:
        xor rax, rax 
        mov [rdi + 0], rax
rawShl_endif3_0:


        
        

        ; Compare with q

        mov rax, [rdi + 0]
        cmp rax, [q + 0]
        jc tmp_111        ; q is bigget so done.
        jnz tmp_110         ; q is lower

        ; If equal substract q
tmp_110:

        mov rax, [q + 0]
        sub [rdi + 0], rax

tmp_111:

        ret
        





;;;;;;;;;;;;;;;;;;;;;;
; shr
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_shr:
        push   rbp
        push   rsi
        push   rdi
        push   rdx
        mov    rbp, rsp


        
        
        
        
        mov    rcx, [rdx]
        bt     rcx, 63          ; Check if is short second operand
        jnc     tmp_112

        ; long 2
        bt     rcx, 62          ; Check if is montgomery second operand
        jnc     tmp_113

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

tmp_113:
        mov rcx, [rdx + 8]
        cmp rcx, 64
        jae  tmp_114
        xor rax, rax
        
        mov rdx, rcx
        jmp do_shr

tmp_114:
        mov rcx, [q]
        sub rcx, [rdx+8]
        cmp rcx, 64
        jae  setzero
        mov rax, [q]
        sub rax, [rdx+8]
        
        mov rdx, rcx
        jmp do_shl

tmp_112:
        cmp ecx, 0
        jl  tmp_115
        cmp ecx, 64
        jae  setzero
        movsx rdx, ecx 
        jmp do_shr
tmp_115:
        neg ecx
        cmp ecx, 64
        jae  setzero
        movsx rdx, ecx 
        jmp do_shl


        

;;;;;;;;;;;;;;;;;;;;;;
; shl
;;;;;;;;;;;;;;;;;;;;;;
; Adds two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result
; Modified Registers:
;    r8, r9, 10, r11, rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_shl:
        push   rbp
        push   rsi
        push   rdi
        push   rdx
        mov    rbp, rsp

        
        
        
        
        mov    rcx, [rdx]
        bt     rcx, 63          ; Check if is short second operand
        jnc     tmp_116

        ; long 2
        bt     rcx, 62          ; Check if is montgomery second operand
        jnc     tmp_117

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

tmp_117:
        mov rcx, [rdx + 8]
        cmp rcx, 64
        jae  tmp_118
        xor rax, rax
        
        mov rdx, rcx
        jmp do_shl

tmp_118:
        mov rcx, [q]
        sub rcx, [rdx+8]
        cmp rcx, 64
        jae  setzero
        mov rax, [q]
        sub rax, [rdx+8]
        
        mov rdx, rcx
        jmp do_shr

tmp_116:
        cmp ecx, 0
        jl  tmp_119
        cmp ecx, 64
        jae  setzero
        movsx rdx, ecx 
        jmp do_shl
tmp_119:
        neg ecx
        cmp ecx, 64
        jae  setzero
        movsx rdx, ecx 
        jmp do_shr



;;;;;;;;;;
;;; doShl
;;;;;;;;;;
do_shl: 
        mov    rcx, [rsi]
        bt     rcx, 63          ; Check if is short second operand
        jc     do_shll
do_shls:

        movsx rax, ecx
        cmp rax, 0
        jz  setzero;
        jl  do_shlcl

        cmp rdx, 31
        jae do_shlcl

        mov cl, dl 
        shl rax, cl
        mov rcx, rax
        shr rcx, 31
        jnz do_shlcl
        mov [rdi], rax
        mov rsp, rbp
        pop   rdx
        pop   rdi
        pop   rsi
        pop   rbp
        ret

do_shlcl:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp do_shlln

do_shll:
        bt      rcx, 62          ; Check if is short second operand
        jnc     do_shlln

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

do_shlln:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d
        add     rdi, 8
        add     rsi, 8
        call    rawShl
        mov rsp, rbp
        pop   rdx
        pop   rdi
        pop   rsi
        pop   rbp
        ret


;;;;;;;;;;
;;; doShr
;;;;;;;;;;
do_shr: 
        mov    rcx, [rsi]
        bt     rcx, 63          ; Check if is short second operand
        jc     do_shrl
do_shrs:
        movsx rax, ecx
        cmp rax, 0
        jz  setzero;
        jl  do_shrcl

        cmp rdx, 31
        jae setzero

        mov cl, dl 
        shr rax, cl
        mov [rdi], rax
        mov rsp, rbp
        pop   rdx
        pop   rdi
        pop   rsi
        pop   rbp
        ret

do_shrcl:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


do_shrl:
        bt      rcx, 62          ; Check if is short second operand
        jnc     do_shrln

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

do_shrln:
        mov r11b, 0x80
        shl r11d, 24
        mov [rdi+4], r11d
        add     rdi, 8
        add     rsi, 8
        call    rawShr
        mov rsp, rbp
        pop   rdx
        pop   rdi
        pop   rsi
        pop   rbp
        ret

setzero:
        xor rax, rax
        mov [rdi], rax
        mov rsp, rbp
        pop   rdx
        pop   rdi
        pop   rsi
        pop   rbp
        ret





;;;;;;;;;;;;;;;;;;;;;;
; rgt - Raw Greater Than
;;;;;;;;;;;;;;;;;;;;;;
; returns in ax 1 id *rsi > *rdx
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rax <= Return 1 or 0
; Modified Registers:
;    r8, r9, rax
;;;;;;;;;;;;;;;;;;;;;;
Fr_rgt:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        mov    r8, [rsi]
        mov    r9, [rdx]
        bt     r8, 63          ; Check if is short first operand
        jc     rgt_l1
        bt     r9, 63          ; Check if is short second operand
        jc     rgt_s1l2

rgt_s1s2:                       ; Both operands are short
        cmp    r8d, r9d
        jg     rgt_ret1
        jmp    rgt_ret0


rgt_l1:
        bt     r9, 63          ; Check if is short second operand
        jc     rgt_l1l2

;;;;;;;;
rgt_l1s2:
        bt     r8, 62          ; check if montgomery first
        jc     rgt_l1ms2
rgt_l1ns2:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp rgtL1L2

rgt_l1ms2:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp rgtL1L2


;;;;;;;;
rgt_s1l2:
        bt     r9, 62          ; check if montgomery second
        jc     rgt_s1l2m
rgt_s1l2n:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp rgtL1L2

rgt_s1l2m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp rgtL1L2

;;;;
rgt_l1l2:
        bt     r8, 62          ; check if montgomery first
        jc     rgt_l1ml2
rgt_l1nl2:
        bt     r9, 62          ; check if montgomery second
        jc     rgt_l1nl2m
rgt_l1nl2n:
        jmp rgtL1L2

rgt_l1nl2m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp rgtL1L2

rgt_l1ml2:
        bt     r9, 62          ; check if montgomery second
        jc     rgt_l1ml2m
rgt_l1ml2n:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp rgtL1L2

rgt_l1ml2m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp rgtL1L2


;;;;;;
; rgtL1L2
;;;;;;

rgtL1L2:


        mov     rax, [rsi + 8]
        cmp     [half + 0], rax     ; comare with (q-1)/2
        jc rgtl1l2_n1                           ; half<rax => e1-e2 is neg => e1 < e2

        jmp rgtl1l2_p1



rgtl1l2_p1:


        mov     rax, [rdx + 8]
        cmp     [half + 0], rax     ; comare with (q-1)/2
        jc rgt_ret1                           ; half<rax => e1-e2 is neg => e1 < e2

        jmp rgtRawL1L2




rgtl1l2_n1:


        mov     rax, [rdx + 8]
        cmp     [half + 0], rax     ; comare with (q-1)/2
        jc rgtRawL1L2                           ; half<rax => e1-e2 is neg => e1 < e2

        jmp rgt_ret0





rgtRawL1L2:

        mov     rax, [rsi + 8]
        cmp     [rdx + 8], rax     ; comare with (q-1)/2
        jc rgt_ret1                      ; rsi<rdx => 1st > 2nd



rgt_ret0:
        xor    rax, rax
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret
rgt_ret1:
        mov    rax, 1
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret



;;;;;;;;;;;;;;;;;;;;;;
; rlt - Raw Less Than
;;;;;;;;;;;;;;;;;;;;;;
; returns in ax 1 id *rsi > *rdx
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rax <= Return 1 or 0
; Modified Registers:
;    r8, r9, rax
;;;;;;;;;;;;;;;;;;;;;;
Fr_rlt:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        mov    r8, [rsi]
        mov    r9, [rdx]
        bt     r8, 63          ; Check if is short first operand
        jc     rlt_l1
        bt     r9, 63          ; Check if is short second operand
        jc     rlt_s1l2

rlt_s1s2:                       ; Both operands are short
        cmp    r8d, r9d
        jl     rlt_ret1
        jmp    rlt_ret0


rlt_l1:
        bt     r9, 63          ; Check if is short second operand
        jc     rlt_l1l2

;;;;;;;;
rlt_l1s2:
        bt     r8, 62          ; check if montgomery first
        jc     rlt_l1ms2
rlt_l1ns2:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp rltL1L2

rlt_l1ms2:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp rltL1L2


;;;;;;;;
rlt_s1l2:
        bt     r9, 62          ; check if montgomery second
        jc     rlt_s1l2m
rlt_s1l2n:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp rltL1L2

rlt_s1l2m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp rltL1L2

;;;;
rlt_l1l2:
        bt     r8, 62          ; check if montgomery first
        jc     rlt_l1ml2
rlt_l1nl2:
        bt     r9, 62          ; check if montgomery second
        jc     rlt_l1nl2m
rlt_l1nl2n:
        jmp rltL1L2

rlt_l1nl2m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp rltL1L2

rlt_l1ml2:
        bt     r9, 62          ; check if montgomery second
        jc     rlt_l1ml2m
rlt_l1ml2n:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp rltL1L2

rlt_l1ml2m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx


        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp rltL1L2


;;;;;;
; rltL1L2
;;;;;;

rltL1L2:


        mov     rax, [rsi + 8]
        cmp     [half + 0], rax     ; comare with (q-1)/2
        jc rltl1l2_n1                           ; half<rax => e1-e2 is neg => e1 < e2

        jmp rltl1l2_p1



rltl1l2_p1:


        mov     rax, [rdx + 8]
        cmp     [half + 0], rax     ; comare with (q-1)/2
        jc rlt_ret0                           ; half<rax => e1-e2 is neg => e1 < e2

        jmp rltRawL1L2




rltl1l2_n1:


        mov     rax, [rdx + 8]
        cmp     [half + 0], rax     ; comare with (q-1)/2
        jc rltRawL1L2                           ; half<rax => e1-e2 is neg => e1 < e2

        jmp rlt_ret1





rltRawL1L2:

        mov     rax, [rsi + 8]
        cmp     [rdx + 8], rax     ; comare with (q-1)/2
        jc rlt_ret0                      ; rsi<rdx => 1st > 2nd
        jnz rlt_ret1


rlt_ret0:
        xor    rax, rax
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret
rlt_ret1:
        mov    rax, 1
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret



;;;;;;;;;;;;;;;;;;;;;;
; req - Raw Eq
;;;;;;;;;;;;;;;;;;;;;;
; returns in ax 1 id *rsi == *rdx
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rax <= Return 1 or 0
; Modified Registers:
;    r8, r9, rax
;;;;;;;;;;;;;;;;;;;;;;
Fr_req:
        push   rbp
        push   rsi
        push   rdx
        mov    rbp, rsp
        mov    r8, [rsi]
        mov    r9, [rdx]
        bt     r8, 63          ; Check if is short first operand
        jc     req_l1
        bt     r9, 63          ; Check if is short second operand
        jc     req_s1l2

req_s1s2:                       ; Both operands are short
        cmp    r8d, r9d
        je     req_ret1
        jmp    req_ret0


req_l1:
        bt     r9, 63          ; Check if is short second operand
        jc     req_l1l2

;;;;;;;;
req_l1s2:
        bt     r8, 62          ; check if montgomery first
        jc     req_l1ms2
req_l1ns2:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toLongNormal
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp reqL1L2

req_l1ms2:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toMontgomery
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp reqL1L2


;;;;;;;;
req_s1l2:
        bt     r9, 62          ; check if montgomery second
        jc     req_s1l2m
req_s1l2n:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toLongNormal
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp reqL1L2

req_s1l2m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toMontgomery
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp reqL1L2

;;;;
req_l1l2:
        bt     r8, 62          ; check if montgomery first
        jc     req_l1ml2
req_l1nl2:
        bt     r9, 62          ; check if montgomery second
        jc     req_l1nl2m
req_l1nl2n:
        jmp reqL1L2

req_l1nl2m:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rdx
        push r8
        call Fr_toMontgomery
        mov  rsi, rdi
        pop  rdi
        pop  rdx

        jmp reqL1L2

req_l1ml2:
        bt     r9, 62          ; check if montgomery second
        jc     req_l1ml2m
req_l1ml2n:

        mov  r8, rdi
        sub  rsp, 16
        mov  rdi, rsp
        push rsi
        mov  rsi, rdx
        push r8
        call Fr_toMontgomery
        mov  rdx, rdi
        pop  rdi
        pop  rsi

        jmp reqL1L2

req_l1ml2m:
        jmp reqL1L2


;;;;;;
; eqL1L2
;;;;;;

reqL1L2:

        mov     rax, [rsi + 8]
        cmp     [rdx + 8], rax
        jne     req_ret0                      ; rsi<rdi => 1st > 2nd


req_ret1:
        mov    rax, 1
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret

req_ret0:
        xor    rax, rax
        mov rsp, rbp
        pop rdx
        pop rsi
        pop rbp
        ret


;;;;;;;;;;;;;;;;;;;;;;
; gt
;;;;;;;;;;;;;;;;;;;;;;
; Compares two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result can be zero or one.
; Modified Registers:
;    rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_gt:
        call Fr_rgt
        mov [rdi], rax
        ret

;;;;;;;;;;;;;;;;;;;;;;
; lt
;;;;;;;;;;;;;;;;;;;;;;
; Compares two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result can be zero or one.
; Modified Registers:
;    rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_lt:
        call Fr_rlt
        mov [rdi], rax
        ret

;;;;;;;;;;;;;;;;;;;;;;
; eq
;;;;;;;;;;;;;;;;;;;;;;
; Compares two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result can be zero or one.
; Modified Registers:
;    rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_eq:
        call Fr_req
        mov [rdi], rax
        ret

;;;;;;;;;;;;;;;;;;;;;;
; neq
;;;;;;;;;;;;;;;;;;;;;;
; Compares two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result can be zero or one.
; Modified Registers:
;    rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_neq:
        call Fr_req
        xor rax, 1
        mov [rdi], rax
        ret

;;;;;;;;;;;;;;;;;;;;;;
; geq
;;;;;;;;;;;;;;;;;;;;;;
; Compares two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result can be zero or one.
; Modified Registers:
;    rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_geq:
        call Fr_rlt
        xor rax, 1
        mov [rdi], rax
        ret

;;;;;;;;;;;;;;;;;;;;;;
; leq
;;;;;;;;;;;;;;;;;;;;;;
; Compares two elements of any kind
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result can be zero or one.
; Modified Registers:
;    rax, rcx
;;;;;;;;;;;;;;;;;;;;;;
Fr_leq:
        call Fr_rgt
        xor rax, 1
        mov [rdi], rax
        ret



;;;;;;;;;;;;;;;;;;;;;;
; rawIsEq
;;;;;;;;;;;;;;;;;;;;;;
; Compares two elements of any kind
; Params:
;   rdi <= Pointer to element 1
;   rsi <= Pointer to element 2
; Returns
;   ax <= 1 if are equal 0, otherwise
; Modified Registers:
;   rax
;;;;;;;;;;;;;;;;;;;;;;
Fr_rawIsEq:

        mov     rax, [rsi + 0]
        cmp     [rdi + 0], rax
        jne     rawIsEq_ret0

rawIsEq_ret1:
        mov    rax, 1
        ret

rawIsEq_ret0:
        xor    rax, rax
        ret

;;;;;;;;;;;;;;;;;;;;;;
; rawIsZero
;;;;;;;;;;;;;;;;;;;;;;
; Compares two elements of any kind
; Params:
;   rdi <= Pointer to element 1
; Returns
;   ax <= 1 if is 0, otherwise
; Modified Registers:
;   rax
;;;;;;;;;;;;;;;;;;;;;;
Fr_rawIsZero:

        cmp     qword [rdi + 0], $0
        jne     rawIsZero_ret0


rawIsZero_ret1:
        mov    rax, 1
        ret

rawIsZero_ret0:
        xor    rax, rax
        ret











;;;;;;;;;;;;;;;;;;;;;;
; land
;;;;;;;;;;;;;;;;;;;;;;
; Logical and between two elements
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result zero or one
; Modified Registers:
;    rax, rcx, r8
;;;;;;;;;;;;;;;;;;;;;;
Fr_land:






    mov     rax, [rsi]
    bt      rax, 63
    jc      tmp_120

    test    eax, eax
    jz      retZero_122
    jmp     retOne_121

tmp_120:

    mov     rax, [rsi + 8]
    test    rax, rax
    jnz     retOne_121


retZero_122:
    mov     qword r8, 0
    jmp     done_123

retOne_121:
    mov     qword r8, 1

done_123:







    mov     rax, [rdx]
    bt      rax, 63
    jc      tmp_124

    test    eax, eax
    jz      retZero_126
    jmp     retOne_125

tmp_124:

    mov     rax, [rdx + 8]
    test    rax, rax
    jnz     retOne_125


retZero_126:
    mov     qword rcx, 0
    jmp     done_127

retOne_125:
    mov     qword rcx, 1

done_127:

        and rcx, r8
        mov [rdi], rcx
        ret


;;;;;;;;;;;;;;;;;;;;;;
; lor
;;;;;;;;;;;;;;;;;;;;;;
; Logical or between two elements
; Params:
;   rsi <= Pointer to element 1
;   rdx <= Pointer to element 2
;   rdi <= Pointer to result zero or one
; Modified Registers:
;    rax, rcx, r8
;;;;;;;;;;;;;;;;;;;;;;
Fr_lor:






    mov     rax, [rsi]
    bt      rax, 63
    jc      tmp_128

    test    eax, eax
    jz      retZero_130
    jmp     retOne_129

tmp_128:

    mov     rax, [rsi + 8]
    test    rax, rax
    jnz     retOne_129


retZero_130:
    mov     qword r8, 0
    jmp     done_131

retOne_129:
    mov     qword r8, 1

done_131:







    mov     rax, [rdx]
    bt      rax, 63
    jc      tmp_132

    test    eax, eax
    jz      retZero_134
    jmp     retOne_133

tmp_132:

    mov     rax, [rdx + 8]
    test    rax, rax
    jnz     retOne_133


retZero_134:
    mov     qword rcx, 0
    jmp     done_135

retOne_133:
    mov     qword rcx, 1

done_135:

        or rcx, r8
        mov [rdi], rcx
        ret


;;;;;;;;;;;;;;;;;;;;;;
; lnot
;;;;;;;;;;;;;;;;;;;;;;
; Do the logical not of an element
; Params:
;   rsi <= Pointer to element to be tested
;   rdi <= Pointer to result one if element1 is zero and zero otherwise
; Modified Registers:
;    rax, rax, r8
;;;;;;;;;;;;;;;;;;;;;;
Fr_lnot:






    mov     rax, [rsi]
    bt      rax, 63
    jc      tmp_136

    test    eax, eax
    jz      retZero_138
    jmp     retOne_137

tmp_136:

    mov     rax, [rsi + 8]
    test    rax, rax
    jnz     retOne_137


retZero_138:
    mov     qword rcx, 0
    jmp     done_139

retOne_137:
    mov     qword rcx, 1

done_139:

        test rcx, rcx

        jz lnot_retOne
lnot_retZero:
        mov qword [rdi], 0
        ret
lnot_retOne:
        mov qword [rdi], 1
        ret


;;;;;;;;;;;;;;;;;;;;;;
; isTrue
;;;;;;;;;;;;;;;;;;;;;;
; Convert a 64 bit integer to a long format field element
; Params:
;   rsi <= Pointer to the element
; Returs:
;   rax <= 1 if true 0 if false
;;;;;;;;;;;;;;;;;;;;;;;
Fr_isTrue:
        





    mov     rax, [rdi]
    bt      rax, 63
    jc      tmp_140

    test    eax, eax
    jz      retZero_142
    jmp     retOne_141

tmp_140:

    mov     rax, [rdi + 8]
    test    rax, rax
    jnz     retOne_141


retZero_142:
    mov     qword rax, 0
    jmp     done_143

retOne_141:
    mov     qword rax, 1

done_143:

        ret





        section .data
Fr_q:
        dd      0
        dd      0x80000000
Fr_rawq:
q       dq      0xffffffff00000001
half    dq      0x7fffffff80000000
R2      dq      0xfffffffe00000001
Fr_R3:
        dd      0
        dd      0x80000000
Fr_rawR3:
R3      dq      0x0000000000000001
lboMask dq      0xffffffffffffffff
np      dq      0xfffffffeffffffff

