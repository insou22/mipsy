# read 10 integers then print them in reverse order

# count in register $s0
# registers $t1 and $t2 used to hold temporary results

main:
    li   $s0, 0           # count = 0

read:
    bge  $s0, 10, print   # while (count < 10) {
    la   $a0, string0     # printf("Enter a number: ");
    li   $v0, 4
    syscall

    li   $v0, 5           #   scanf("%d", &numbers[count]);
    syscall               #
    mul  $t1, $s0, 4      #   calculate &numbers[count]
    la   $t2, numbers     #
    add  $t1, $t1, $t2    #
    sw   $v0, ($t1)       #   store entered number in array

    add  $s0, $s0, 1      #   count++;
    b read                # }

print:
    la   $a0, string1     # printf("Reverse order:\n");
    li   $v0, 4
    syscall
    li   $s0, 9           # count = 9;
next:
    blt  $s0, 0, end1     # while (count >= 0) {

    mul  $t1, $s0, 4      #   printf("%d", numbers[count])
    la   $t2, numbers     #   calculate &numbers[count]
    add  $t1, $t1, $t2    #
    lw   $a0, ($t1)       #   load numbers[count] into $a0
    li   $v0, 1
    syscall

    li   $a0, '\n'        #   printf("%c", '\n');
    li   $v0, 11
    syscall

    sub  $s0, $s0,1       #   count--;
    b next                # }
end1:

    li   $v0, 0           # return 0
    jr   $ra

.data

numbers:                 # int numbers[10];
     .word 0 0 0 0 0 0 0 0 0 0

string0:
    .asciiz "Enter a number: "
string1:
    .asciiz "Reverse order:\n"
