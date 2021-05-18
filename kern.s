.ktext
_start:
    la      $k0, main
    jalr    $k0

    la      $k0, kernel__v0
    sw      $v0, ($k0)
    lw      $a0, ($k0)
    li      $v0, 17
    syscall

.kdata
    kernel__v0: .space 4
