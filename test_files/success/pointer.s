# demonstrate implementation of pointers by an address
# p in register $t0
# i in register $t1
# $t2 used for temporary value
main:
    la   $t0, answer    # p = &answer;

    lw   $t1, ($t0)     # i = *p;

    move $a0, $t1       # printf("%d\n", i);
    li   $v0, 1
    syscall

    li   $a0, '\n'      # printf("%c", '\n');
    li   $v0, 11
    syscall

    li   $t2, 27        # *p = 27;
    sw   $t2, ($t0)     #

    lw   $a0, answer    # printf("%d\n", answer);
    li   $v0, 1
    syscall

    li   $a0, '\n'      # printf("%c", '\n');
    li   $v0, 11
    syscall

    li   $v0, 0         # return 0 from function main
    jr   $ra            #

    .data
answer:
    .word 42            # int answer = 42;
