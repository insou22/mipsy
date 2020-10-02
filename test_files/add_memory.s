# add 17 and 25 use variables stored in memory and print result

main:                  #  x, y, z in $t0, $t1, $t2,
    li   $t0, 17       # x = 17;
    sw   $t0, x

    li   $t0, 25       # y = 25;
    sw   $t0, y

    lw   $t0, x
    lw   $t1, y
    add  $t2, $t1, $t0 # z = x + y
    sw   $t2, z

    lw   $a0, z       # printf("%d", a0);
    li   $v0, 1
    syscall

    li   $a0, '\n'    # printf("%c", '\n');
    li   $v0, 11
    syscall

    li   $v0, 0       # return 0
    jr   $ra

.data
x:  .word 0
y:  .word 0
z:  .word 0
