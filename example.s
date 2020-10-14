main:
	li		$t0, 0			# int counter = 0;

loop:
	bge		$t0, 42, end	# while (count < 42) {

	li		$a0, 42			#     printf("%d", 42);
	li		$v0, 1
	syscall

	li		$a0, '\n'		#     printf("%c", '\n');
	li		$v0, 11
	syscall

	addiu	$t0, $t0, 1		#     counter++;
	b		loop			# }

end:
	jr		$ra

