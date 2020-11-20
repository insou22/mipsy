main:
    li   $t0, 0x03040506

    sw   $t0, u

    lb   $a0, u

    li   $v0, 1         # printf("%d", a0);

    syscall

    li   $a0, '\n'      # printf("%c", '\n');
    li   $v0, 11
    syscall


    li   $v0, 0          # return 0
    jr   $ra

    .data
u:
    .word 0
