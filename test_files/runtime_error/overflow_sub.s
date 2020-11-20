    .text
main:
    li      $t0, -0x7FFFFFFF
    li      $t1, 42

    sub     $t2, $t0, $t1

    li      $v0, 0
    jr      $ra