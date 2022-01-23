    .data
foo: .byte 1, 2, 3:3, 4, 5, 6:3

    .text
main:
    li      $t0, 0

loop:
    bge     $t0, 10, end

    li      $v0, 1
    lb      $a0, foo($t0)
    syscall

    li      $v0, 11
    li      $a0, '\n'
    syscall

    addiu   $t0, 1
    b       loop

end:
    jr      $ra
