# instruction add on line 10 is spelt with three d's

# add 17 and 25  and print result

main:                    #  x, y, z in $t0, $t1, $t2,
    li   $t0, 17         # x = 17;

    li   $t1, 25         # y = 25;

    addd  $t2, $t1, $t0   # z = x + y

    move $a0, $t2        # printf("%d", a0);
    li   $v0, 1
    syscall

    li   $a0, '\n'       # printf("%c", '\n');
    li   $v0, 11
    syscall

    li   $v0, 0          # return 0
    jr   $ra
