    .text
main:
    li      $t0, 42
    li      $t1, 0

    div     $t2, $t0, $t1

    li      $v0, 0
    jr      $ra