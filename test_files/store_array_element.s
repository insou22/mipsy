main:
    li   $t0, 3
    mul  $t0, $t0, 4
    la   $t1, x
    add  $t2, $t1, $t0
    li   $t3, 17
    sw   $t3, 0($t2)
    # ...
.data
x:  .space 40
