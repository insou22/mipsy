main:
    la   $a0, string  # get addr of string
    li   $v0, 4       # 4 is print string syscall
    syscall
    jr   $ra

    .data
string:
    .asciiz "I love MIPS\n

