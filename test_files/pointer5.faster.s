# print 5 numbers -
# this is closer to the code a compiler might produce
# p in $s0
# q in $s1

main:
    la   $s0, numbers    # int *p = &numbers[0];
    add  $s1, $s0, 16    # int *q = &numbers[4];
loop:
    lw   $a0, ($s0)      # printf("%d", *p);
    li   $v0, 1
    syscall
    li   $a0, '\n'       #   printf("%c", '\n');
    li   $v0, 11
    syscall
    add  $s0, $s0, 4     #   p++
    ble  $s0, $s1, loop  # if (p <= q) goto loop;

    li   $v0, 0          # return 0
    jr   $ra

.data

numbers:                 # int numbers[10] = { 3, 9, 27, 81, 243};
     .word 3, 9, 27, 81, 243
