# print 5 numbers
# p in $s0
# q in $s1
# j in $s2

main:
    la   $s0, numbers    # int *p = &numbers[0];
    la   $t0, numbers    # int *q = &numbers[4];
    add  $s1, $t0, 16    #
loop:
    bgt  $s0, $s1, end   # if (p > q) goto end;
    lw   $s2, ($s0)      # int j = *p;
    move $a0, $s2        # printf("%d", j);
    li   $v0, 1
    syscall
    li   $a0, '\n'       #   printf("%c", '\n');
    li   $v0, 11
    syscall

    add  $s0, $s0, 4     #   p++
    b loop               # goto loop
end:

    li   $v0, 0          # return 0
    jr   $ra

.data

numbers:                 # int numbers[10] = { 3, 9, 27, 81, 243};
     .word 3, 9, 27, 81, 243
