# example of function calls
# note storing of return  address $a0, $a1
# and $ra on stack
# for simplicity we are not using a frame pointer

main:
    sub  $sp, $sp, 4    # move stack pointer down to make room
    sw   $ra, 0($sp)    # save $ra on $stack

    li   $a0, 10         # sum_product(10, 12);
    li   $a1, 12
    jal  sum_product

    move $a0, $v0       # printf("%d", z);
    li   $v0, 1
    syscall

    li   $a0, '\n'      # printf("%c", '\n');
    li   $v0, 11
    syscall

    lw   $ra, 0($sp)    # recover $ra from $stack
    add  $sp, $sp, 4    # move stack pointer back up to what it was when main called

    li   $v0, 0         # return 0 from function main
    jr   $ra            # return from function main



sum_product:
    sub  $sp, $sp, 12   # move stack pointer down to make room
    sw   $ra, 8($sp)    # save $ra on $stack
    sw   $a1, 4($sp)    # save $a1 on $stack
    sw   $a0, 0($sp)    # save $a0 on $stack

    li   $a0, 6         # product(6, 7);
    li   $a1, 7
    jal  product

    lw   $a1, 4($sp)    # restore $a1 from $stack
    lw   $a0, 0($sp)    # restore $a0 from $stack

    add  $v0, $v0, $a0  # add a and b to value returned in $v0
    add  $v0, $v0, $a1  # and put result in $v0 to be returned

    lw   $ra, 8($sp)    # restore $ra from $stack
    add  $sp, $sp, 12   # move stack pointer back up to what it was when main called

    jr   $ra            # return from sum_product


product:                # product doesn't call other functions
                        # so it doesn't need to save any registers
    mul  $v0, $a0, $a1  # return argument * argument 2
    jr   $ra            #
