# simple example of placing return  address on stack
# note stack grows down

main:
    sub  $sp, $sp, 4    # move stack pointer down to make room
    sw   $ra, 0($sp)    # save $ra on $stack

    la   $a0, string0   # printf("calling function f\n");
    li   $v0, 4
    syscall

    jal  f              # set $ra to following address

    la   $a0, string1   # printf("back from function f\n");
    li   $v0, 4
    syscall

    lw   $ra, 0($sp)    # recover $ra from $stack
    add  $sp, $sp, 4    # move stack pointer back to what it was

    li   $v0, 0         # return 0 from function main
    jr   $ra            #


f:
    la $a0, string2     # printf("in function f\n");
    li $v0, 4
    syscall
    jr $ra              # return from function f


    .data
string0:
    .asciiz "calling function f\n"
string1:
    .asciiz "back from function f\n"
string2:
    .asciiz "in function f\n"
