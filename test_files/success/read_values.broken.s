# read a number and print whther its odd or even
#
main:
    la   $a0, string0    # printf("Enter an int: ");
    li   $v0, 4
    syscall

    li   $v0, 5          # scanf("%d", x);
    syscall

    move $t0, $v0         # t0 = x

    la   $a0, string2    # printf("Enter a char: ");
    li   $v0, 4
    syscall

    li   $v0, 12          # scanf("%c", x);
    syscall

    move $t5, $v0         # t0 = x

    la   $a0, string3    # printf("Enter a String of 64 bytes: ");
    li   $v0, 4
    syscall


    la   $a0, user_input
    li   $a1, 64
    li   $v0, 8          # scanf("%s[64]", x);
    syscall

    # Scan Float
    # la   $a0, string0    # printf("Enter a float: ");
   #  li   $v0, 4
   #  syscall

   #  li   $v0, 6          # scanf("%f", x);
   #  syscall

   #  move $t1, $v0         # t0 = x
   #  
   #  # Scan Double
   #  la   $a0, string0    # printf("Enter a double: ");
   #  li   $v0, 4
   #  syscall

   #  li   $v0, 7          # scanf("%lf", x);
   #  syscall

   #  move $t3, $f0         # t0 = x
   #  move $t4, $f1          


    # print int
    move $a0, $t0 
    li   $v0, 1
    syscall

    la   $a0, string1
    li   $v0, 4
    syscall

    #print char
    move $a0, $t5
    li $v0, 11
    syscall

    la   $a0, string1
    li   $v0, 4
    syscall


    la $a0, user_input
    li $v0, 4
    syscall

    la   $a0, string1
    li   $v0, 4
    syscall
    # print float
    # move $f12, $t1
    # li   $v0, 2
    # syscall

    # la   $a0, string1
    # li   $v0, 4
    # syscall

    # print double
    #move $f12, $t3
    #move $f12, $t4
    #li   $v0, 2
    #syscall

end:
    li   $v0, 0          # return 0
    jr   $ra

    .data

user_input: 
    .space 64

string0:
    .asciiz "Enter an int: "

string1:
    .asciiz "\n"

string2:
    .asciiz "Enter a char: "

string3:
    .asciiz "Enter a string: "
