# reads in a bunch of different datatypes
.data
    newline: .asciiz "\n"
    userInput: .space 20

.text

main:

    # read in a single integer and print it
    li $v0, 5
    syscall
    
    move $t0, $v0
    li $v0, 1
    move $a0, $t0
    syscall
    


    # read in a single char, print it
    li $v0, 12
    syscall

    move $t0, $v0
    li $v0, 11
    move $a0, $t0
    syscall


    #read in a string and print it

    li $v0, 8
    li $a0, userInput
    li $a1, 20
    syscall

    li $v0, 4
    li $a0, userInput
    syscall

    li $v0, 0
    jr $ra
