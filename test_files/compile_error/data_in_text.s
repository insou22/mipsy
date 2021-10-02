foo:
    .asciiz "hello!\n"

main:
    li  $v0, 4
    la  $a0, foo
    syscall

    jr  $ra
