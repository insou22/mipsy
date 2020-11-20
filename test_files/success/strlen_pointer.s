# simple example of placing return  address $ra and $a0 on stack
# for simplicity we are not using a frame pointer

main:
    sub $sp, $sp, 4     # move stack pointer down to make room
    sw $ra, 0($sp)      # save $ra on $stack

    la $a0, string      # my_strlen("Hello Andrew");
    jal my_strlen

    move $a0, $v0       # printf("%d", i);
    li $v0, 1
    syscall

    li $a0, '\n'        # printf("%c", '\n');
    li $v0, 11
    syscall

    lw $ra, 0($sp)      # recover $ra from $stack
    add $sp, $sp, 4     # move stack pointer back up to what it was when main called

    jr $ra              # return from function main

my_strlen:              # length in t0, s in $a0
    li $t0, 0
loop:                   #
    lb $t1, 0($a0)      # load *s into $t1
    beq $t1, 0, end     #
    add $t0, $t0, 1     # length++
    add $a0, $a0, 1     # s++
    b loop              #
end:
    move $v0, $t0       # return length
    jr $ra

    .data
string:
    .asciiz "Hello Andrew"