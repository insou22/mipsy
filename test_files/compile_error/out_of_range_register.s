# tries to use $32 on line 10

# li on line 20 doesn't provide first register argument

# add 17 and 25  and print result

main:                    #  x, y, z in $t0, $t1, $t2,
    li   $t0, 17         # x = 17;

    li   $32, 25         # y = 25;

    add  $t2, $t1, $t0   # z = x + y

    move $a0, $t2        # printf("%d", a0);
    li   $v0, 1
    syscall

    li   $a0, '\n'       # printf("%c", '\n');
    li   $v0, 11
    syscall

    li   0          # return 0
    jr   $ra
