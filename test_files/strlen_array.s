# calculate the  length of a string using a strlen like function

main:
    sub  $sp, $sp, 4    # move stack pointer down to make room
    sw   $ra, 0($sp)    # save $ra on $stack

    la   $a0, string    # my_strlen("Hello Andrew");
    jal  my_strlen

    move $a0, $v0       # printf("%d", i);
    li   $v0, 1
    syscall

    li   $a0, '\n'      # printf("%c", '\n');
    li   $v0, 11
    syscall

    lw   $ra, 0($sp)    # recover $ra from $stack
    add  $sp, $sp, 4    # move stack pointer back up to what it was when main called

    li   $v0, 0         # return 0 from function main
    jr   $ra            #


my_strlen:              # length in t0, s in $a0
    li   $t0, 0
loop:                   # while (s[length] != 0) {
    add  $t1, $a0, $t0  #   calculate &s[length]
    lb   $t2, 0($t1)    #   load s[length] into $t2
    beq  $t2, 0, end    #
    add  $t0, $t0, 1    #   length++;
    b    loop           # }
end:
    move $v0, $t0       # return length
    jr   $ra

    .data
string:
    .asciiz "Hello Andrew"