; ModuleID = 'fr.ll'
source_filename = "fr.ll"

define i128 @fr_add(i128 %0, i128 %1) {
main:
    %add = add i128 %0, %1
    ret i128 %add
}

define i128 @fr_mul(i128 %0, i128 %1) {
main:
    %mul = mul i128 %0, %1
    ret i128 %mul
}