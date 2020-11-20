# print integers 1..10 one per line

main:                 # int main(void) {
                      # int i;  // in register $t0

    li    $t0, 1      # i = 1;

loop:                 # loop:
    bgt  $t0, 10, end  # if (i > 10) goto end;

    move $a0, $t0     #   printf("%d" i);
    li   $v0, 1
    syscall

    li   $a0, '\n'      # printf("%c", '\n');
    li   $v0, 11
    syscall

    add  $t0, $t0, 1   #   i++;

    b    loop         # goto loop;

end:
    li   $v0, 0       # return 0
    jr   $ra
