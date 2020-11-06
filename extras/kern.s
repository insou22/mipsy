main:
    lui     $k0, 0
    ori     $k0, $k0, 0
    jalr    $k0
    move    $a0, $v0
    li      $v0, 17
    syscall