main:
	li	$v0, 10
	seq	$v0, $zero, $v0

	move	$a0, $v0
	li	$v0, 1
	syscall

	li	$a0, '\n'
	li	$v0, 11
	syscall

	li	$v0, 0
	jr	$ra
