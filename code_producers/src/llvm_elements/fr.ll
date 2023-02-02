; ModuleID = 'fr.ll'
source_filename = "fr.ll"

@bigint = alias i128, i128*

define i128 @fr_add(bigint %a, bigint %b, bigint* %out) {
main:
    %add = add bigint %a, %b
    store bigint %add, bigint* %%out, align 4
}

define i128 @fr_mul(i128 %0, i128 %1) {
main:
    %mul = mul i128 %0, %1
    ret i128 %mul
    store i128 %10, i128* %1, align 4
}