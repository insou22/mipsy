# read 10 numbers into an array then print the 10 numbers

# i in register $s0
# registers $t1, $t2 & $t3 used to hold temporary results

main:

    li $s0, 0           # i = 0
loop0:
    bge $s0, 10, end0   # while (i < 10) {

    la $a0, string0     #   printf("Enter a number: ");
    li $v0, 4
    syscall

    li $v0, 5           #   scanf("%d", &numbers[i]);
    syscall             #

    mul $t1, $s0, 4     #   calculate &numbers[i]
    la $t2, numbers     #
    add $t3, $t1, $t2   #
    sw $v0, ($t3)       #   store entered number in array

    add $s0, $s0, 1     #   i++;
    b loop0             # }
end0:

    li   $s0, 0          # i = 0
loop1:
    bge  $s0, 10, end1   # while (i < 10) {

    mul  $t1, $s0, 4     #   calculate &numbers[i]
    la   $t2, numbers    #
    add  $t3, $t1, $t2   #
    lw   $a0, ($t3)      #   load numbers[i] into $a0
    li   $v0, 1          #   printf("%d", numbers[i])
    syscall

    li   $a0, '\n'       #   printf("%c", '\n');
    li   $v0, 11
    syscall

    add  $s0, $s0, 1     #   i++
    b loop1              # }
end1:

    li   $v0, 0          # return 0
    jr   $ra

.data

numbers:                # int numbers[10];
     .word 0, 0, 0, 0, 0, 0, 0, 0, 0, 0

string0:
    .asciiz "Enter a number: "
