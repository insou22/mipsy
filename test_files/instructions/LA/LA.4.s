main:
	li	$a0, 1
	la	$a0, 0($a0)
	li	$v0, 1
	syscall

	li	$a0, '\n'
	li	$v0, 11
	syscall

	li	$v0, 0
	jr	$ra
