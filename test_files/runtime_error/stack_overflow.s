main:
    jal     foo

foo:
    addiu   $sp, $sp, -4
    sw      $ra, ($sp)

    j       bar

    lw      $ra, ($sp)
    addiu   $sp, $sp, 4
    
    jr      $ra

bar:
    jr      $ra
