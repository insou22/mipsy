    .text
main:
    lw      $t0, my_variable
    li      $v0, 1
    move    $a0, $t0
    syscall             # should print 42 ???

    li      $v0, 0
    jr      $ra

    .data
my_variable: .space 4 .word 42