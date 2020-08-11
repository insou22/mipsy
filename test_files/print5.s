# print 5 numbers
# i in $s0
# j in $s1

main:
    li   $s0, 0          # int i = 0;
loop:
    bge  $s0, 5, end     # if (i >= 5) goto end;
    la   $t0, numbers    #    int j = numbers[i];
    mul  $t1, $s0, 4
    add  $t2, $t1, $t0
    lw   $s1, ($t2)
    move $a0, $s1        # printf("%d", j);
    li   $v0, 1
    syscall
    li   $a0, '\n'       #   printf("%c", '\n');
    li   $v0, 11
    syscall

    add  $s0, $s0, 1     #   i++
    b loop               # goto loop
end:

    li   $v0, 0          # return 0
    jr   $ra

.data

numbers:                 # int numbers[10] = { 3, 9, 27, 81, 243};
     .word 3, 9, 27, 81, 243
