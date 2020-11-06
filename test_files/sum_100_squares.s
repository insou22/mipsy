# calculate 1*1 + 2*2 + ... + 99 * 99 + 100 * 100

# sum in $t0, i in $t1

main:
    li  $t0, 0          # sum = 0;
    li  $t1, 0          # i = 0

loop:
    bgt $t1, 100, end    # if (i > 100) goto end;
    mul $t3, $t1, $t1   # t3 = i * i;
    add $t0, $t0, $t3   # sum = sum + t3;

    add $t1, $t1, 1     # i = i + 1;
    b   loop

end:
    move $a0, $t0      # printf("%d", sum);
    li   $v0, 1
    syscall

    li   $a0, '\n'      # printf("%c", '\n');
    li   $v0, 11
    syscall

    li   $v0, 0         # return 0
    jr   $ra
