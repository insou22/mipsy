# using a frame pointer to handle stack growing during function execution

f:
    sub  $sp, $sp, 12   # move stack pointer down to make room
    sw   $fp, 8($sp)    # save $fp on $stack
    sw   $ra, 4($sp)    # save $ra on $stack
    sw   $a0, 0($sp)    # save $a0 on $stack
    add  $fp, $sp, 12   # have frame pointer at start of stack frame

    li   $v0, 5         # scanf("%d", &length);
    syscall

    mul  $v0, $v0, 4    # calculate array size
    sub  $sp, $sp, $v0  # move stack_pointer down to hold array

    # ... more code ...

    lw   $ra, -8($fp)   # restore $ra from stack
    move $sp, $fp       # move stack pointer backup  to what it was when main called
    lw   $fp, -4($fp)   # restore $fp from $stack
    jr   $ra            # return

