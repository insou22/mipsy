# a bug in older versions of mipsy; now a regression test.
# Author: Stephen Vinall <s.vinall@unswglobal.unsw.edu.au>

# mipsy: The "bgezal" (branch and link if greater than or equal to zero)
# and "bltzal" instructions are supposed to unconditionally overwrite $ra
# with the return address whether or not the branch was taken (!).

# This behaviour is correct in spim/qtspim. However, mipsy only sets $ra 
# on a successful branch. The "MIPS32-II-r5.04.pdf" instruction reference 
# for "bgezal" (page 73) shows pseudocode that confirms GPR[31] ($ra) should 
# be set whether or not the branch is taken.

main:
        move    $t3, $ra        # save $ra
        li      $t1, -1
        bgezal  $t1, main       # <--- instruction being tested
                                # (does not branch since -1 < 0)
        sne     $t2, $ra, $t3   # $t2 = 1 if $ra has changed (expected)

        la      $t0, msg_ok
        la      $t1, msg_error
        li      $v0, 4
        movn    $a0, $t0, $t2   # msg_ok    if $t2 != 0
        movz    $a0, $t1, $t2   # msg_error if $t2 == 0
        syscall

        li      $v0, 0
        move    $ra, $t3        # restore $ra
        jr      $ra

        .data
msg_ok:
        .asciiz "ok: bgezal changed $ra unconditionally\n"
msg_error:
        .asciiz "error: bgezal did not change $ra unconditionally\n"
