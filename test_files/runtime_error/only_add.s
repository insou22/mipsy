# I genuinely don't know what this was supposed to do
# Uninitialised register $s1 on line 6 I suppose!

	.text
main:
	add		$s0, $s1, $s2
	add		$s0, $s0, $s3

	.data
words:
	.word 0, 1, 2, 3, 4

bytes:
	.byte 5, 6, 7, 8, 9

hello:
	.asciiz "Hello"
