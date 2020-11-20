# i in register $s0
# registers $s1 and $s2 used to hold temporary results
main:
    li $s0, 0           # i = 0

loop0:
    bge $s0, 10, end0   # while (i < 10) {
    la $a0, string0     # printf("Enter a number: ");
    li $v0, 4
    syscall

    li $v0, 5           # scanf("%d", &numbers[i]);
    syscall             #
    mul $s1, $s0, 4     # calculate &numbers[i]
    la $s2, numbers     #
    add $s1, $s1, $s2   #
    sw $v0, ($s1)       # store entered number in array

    add $s0, $s0, 1     # i++;
    b loop0
end0:
    li $s0, 0           # i = 0

loop1:
    bge $s0, 10, done   # while (i < 10) {

    mul $s1, $s0, 4     # printf("%d", numbers[i])
    la $s2, numbers     # calculate &numbers[i]
    add $s1, $s1, $s2   #
    lw $a0, ($s1)       # load numbers[i] into $a0
    li $v0, 1
    syscall

    li   $a0, '\n'      # printf("%c", '\n');
    li   $v0, 11
    syscall

    add $s0, $s0, 1     # i++
    b loop1

done:
    jr $31

.data

numbers:
    .space 40           # int numbers[10];

string0:
    .asciiz "Enter a number: "
string1:
    .asciiz "Reverse order:\n"
