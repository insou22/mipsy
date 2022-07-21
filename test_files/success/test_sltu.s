# a bug in older versions of mipsy; now a regression test.
# Author: Stephen Vinall <s.vinall@unswglobal.unsw.edu.au>

# mipsy: "sltu" is incorrectly interpreting it's arguments as signed
# (not unsigned). This means the pseudo-instructions "bgtu", "bgeu",
# etc. which generate "sltu" are incorrectly doing signed comparisons.

main:
        li      $t0, 1
        li      $t1, -1
        sltu    $t2, $t0, $t1   # <--- instruction being tested

        # $t2 should be 1 (true) because: 0x1 < 0xffffffff (unsigned)
        # mipsy returns 0 (false) because it incorrectly sees: 1 < -1 (signed)

        la      $t0, msg_ok
        la      $t1, msg_error
        li      $v0, 4
        movn    $a0, $t0, $t2    # msg_ok    if $t2 != 0
        movz    $a0, $t1, $t2    # msg_error if $t2 == 0
        syscall

        li      $v0, 0
        jr      $ra

        .data
msg_ok:
        .asciiz "ok: sltu interpreted the arguments as unsigned\n"
msg_error:
        .asciiz "error: sltu interpreted the arguments as signed\n"
