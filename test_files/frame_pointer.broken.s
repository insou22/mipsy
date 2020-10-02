# example stack growing during function execution
# breaking the function return

f:
    sub  $sp, $sp, 8     # move stack pointer down to make room
    sw   $ra, 4($sp)     # save $ra on $stack
    sw   $a0, 0($sp)     # save $a0 on $stack

    li   $v0, 5          # scanf("%d", &length);
    syscall

    mul  $v0, $v0, 4     # calculate array size
    sub  $sp, $sp, $v0   # move stack_pointer down to hold array

    # ...

                        # breaks because stack pointer moved down to hold array
                        # so we won't restore the correct value
    lw   $ra, 4($sp)    # restore $ra from $stack
    add  $sp, $sp, 8    # move stack pointer back up to what it was when main called

    jr   $ra            # return from f

