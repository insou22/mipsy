    .text
main:
    li      $t0, 1
    li      $t1, 2
    li      $t2, 3

    add     $t0, $t0, $t1
    add     $t1, $t2, $t3
    add     $t2, $t3, $t4

    li      $v0, 0
    jr      $ra