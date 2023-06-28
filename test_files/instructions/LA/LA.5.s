CONSTANT = 0

main:
	li	$a0, 1
	la	$a0, 32768($a0)
	li	$v0, 1
	syscall

	li	$a0, '\n'
	li	$v0, 11
	syscall

	li	$v0, 0
	jr	$ra
