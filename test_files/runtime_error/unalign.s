# poor alignment

main:
    li $t0, 1

    sb $t0, v1            # will succeed because no alignment needed
    sh $t0, v1            # will fail because v1 is not aligned on 2-byte boundary
    sw $t0, v1            # will fail because v1 is not aligned on 4-byte boundary

    sh $t0, v2            # will succeeed because v2 is aligned on 2-byte boundary
    sw $t0, v2            # will fail because v2 is not aligned on a 4-byte boundary

    sh $t0, v3            # will succeeed because v3 is aligned on 2-byte boundary
    sw $t0, v3            # will fail because v3 is not aligned on a 4-byte boundary

    sh $t0, v4            # will succeeed because v4 is aligned on 2-byte boundary
    sw $t0, v4            # will succeeed because v4 is  aligned on a 4-byte boundary

    sw $t0, v5            # will succeeed because v5 is aligned on a 4-byte boundary

    sw $t0, v6            # will succeeed because v6 is aligned on a 4-byte boundary

    jr   $ra             # return

    .data     # data will be aligned on a 4-byte boundary
              # most likely on at least a 128-byte boundary
              # but safer to just add a .align directive
    .align 4
    .space 1
v1:
    .space 1
v2:
    .space 4
v3:
    .space 2
v4:
    .space 4
    .space 1
    .align 2 # ensure e is on a 4 (2**2) byte boundary
v5:
    .space 4
    .space 1
v6:
    .word 0  # word directive automaticatically aligns on 4 byte boundary
