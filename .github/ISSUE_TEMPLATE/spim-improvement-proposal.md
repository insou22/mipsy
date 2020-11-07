---
name: SPIM Improvement Proposal
about: Something that can be improved in SPIM for Mipsy
title: "[SIP] Division by zero"
labels: SPIM Improvement
assignees: insou22

---

<!--  Brief explanation of SPIM fault  -->
SPIM's default exception message for a division by zero is extremely poor for students.


# Example

<!-- 
    Example of how to reproduce this behaviour in SPIM.
    Feel free to delete example section and replace with "No relevant examples" if needed.
-->

## MIPS input:
```
main:
	div		$t0, $0, $0
	jr		$ra
```

## SPIM output:
```
❯ spim -f foo.s
Loaded: /home/zac/uni/teach/comp1521/20T2/work/spim-simulator/CPU/exceptions.s
Exception occurred at PC=0x00400028
  Exception 9  [Breakpoint]  occurred and ignored
```

<!--  What should Mipsy aim to do to improve this behaviour for students?  -->
# Suggested improvements

- Explain that the error was a division by zero
- Tell the student the instruction responsible
- Display the context of the error - eg. the registers involved, and their current values

<!--  Any other notes you feel are necessary  -->
# Other notes?