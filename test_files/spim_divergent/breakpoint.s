# SPIM does not recognise the `break` instruction
# (R-type, args=[], function = 0b001101)
# Mipsy does, and will:
#  - ignore it in normal-mode
#  - recognise it as a breakpoint in interactive-mode

main:
	break
	li		$v0, 1
	break
	li		$a0, 42
	break
	syscall
	break
	jr		$ra
