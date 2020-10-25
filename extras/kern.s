main:
    li      $k0, 0x00400000
    jalr    $k0
    move    $a0, $v0
    li      $v0, 17
    syscall