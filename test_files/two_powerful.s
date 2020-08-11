# simple example of placing return  address $ra and $a0 on stack
# for simplicity we are not using a frame pointer

main:
    sub $sp, $sp, 4     # move stack pointer down to make room
    sw $ra, 0($sp)      # save $ra on $stack

    li $a0, 1           # two(1);
    jal two


    lw $ra, 0($sp)      # recover $ra from $stack
    add $sp, $sp, 4     # move stack pointer back up to what it was when main called

    jr $ra              # return from function main



two:
    sub $sp, $sp, 8     # move stack pointer down to make room
    sw $ra, 4($sp)      # save $ra on $stack
    sw $a0, 0($sp)      # save $a0 on $stack

    bge $a0, 1000000, print
    mul $a0, $a0, 2     # restore $a0 from $stack
    jal two
print:

    lw $a0, 0($sp)      # restore $a0 from $stack
    li $v0, 1           # printf("%d");
    syscall

    li $a0, '\n'        # printf("%c", '\n');
    li $v0, 11
    syscall

    lw $ra, 4($sp)      # restore $ra from $stack
    add $sp, $sp, 8     # move stack pointer back up to what it was when main called

    jr $ra              # return from two
