print:
    li      $v0, 1
    li      $a0, 42
    syscall

    jr      $ra

main:
    move    $s0, $ra

    jal     print
    jal     print
    jal     print
    
    move    $ra, $s0
    jr      $ra