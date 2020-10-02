# print a 2d array
# i in $s0
# j in $s1

main:
    li   $s0, 0          # int i = 0;
loop1:
    bge  $s0, 3, end1    # if (i >= 3) goto end1;
    li   $s1, 0          #    int j = 0;
loop2:
    bge  $s1, 5, end2    #    if (j >= 5) goto end2;
    la   $t0, numbers    #        printf("%d", numbers[i][j]);
    mul  $t1, $s0, 20
    add  $t2, $t1, $t0
    mul  $t3, $s1, 4
    add  $t4, $t3, $t2
    lw   $a0, ($t4)
    li   $v0, 1
    syscall
    li   $a0, ' '       #       printf("%c", ' ');
    li   $v0, 11
    syscall
    add  $s1, $s1, 1     #       j++;
    b loop2              #    goto loop2;
end2:
    li   $a0, '\n'       #    printf("%c", '\n');
    li   $v0, 11
    syscall

    add  $s0, $s0, 1      #   i++
    b loop1               # goto loop1
end1:

    li   $v0, 0          # return 0
    jr   $ra

.data
# int numbers[3][5] = {{3,9,27,81,243},{4,16,64,256,1024},{5,25,125,625,3125}};
numbers:
     .word  3, 9, 27, 81, 243, 4, 16, 64, 256, 1024, 5, 25, 125, 625, 3125
