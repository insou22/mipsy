# MIPS32 - Release 5 Instructions:
main:

### ABS ### - Absolute Value of Word - Arithmetic Instruction [Psuedo-Instruction]
ABS	$s0, $t1
ABS	$s0
# ABS	$s0, 10
# ABS	$s0, -10
# ABS	$s0, 2100000000
# ABS	$s0, -2100000000
# ABS	$s0, 4000000000


### ADD ### - Add Word - Arithmetic Instruction
ADD	$s0, $t1, $t2                 # $rd = $rs + $rt <real instruction>
ADD	$s0, $t1                      # $rd = $rd + $rs [warn: pseudo-instruction: inplace shorthand]
ADD	$s0, $t1, 10                  # $rd = $rs + i16 [warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, 10                       # $rd = $rd + i16 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, 10, $t2                  # $rd = i16 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, $t1, -10                 # $rd = $rs + i16 [warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, -10                      # $rd = $rd + i16 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, -10, $t2                 # $rd = i16 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, $t1, 2100000000          # $rd = $rs + i32 [warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, 2100000000               # $rd = $rd + i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, 2100000000, $t2          # $rd = i32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, $t1, -2100000000         # $rd = $rs + i32 [warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, -2100000000              # $rd = $rd + i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, -2100000000, $t2         # $rd = i32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, $t1, 4000000000          # $rd = $rs + u32 [warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, 4000000000               # $rd = $rd + u32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADD	$s0, 4000000000, $t2          # $rd = u32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]

### ADDI ### - Add Immediate Word - Arithmetic Instruction
# ADDI	$s0, $t1, $t2                 # $rd = $rs + $rt [warn: pseudo-instruction: is an immidiate instruction]
# ADDI	$s0, $t1                      # $rd = $rd + $rs [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: is an immidiate instruction]
ADDI	$s0, $t1, 10                  # $rd = $rs + i16 <real instruction>
ADDI	$s0, 10                       # $rd = $rd + i16 [warn: pseudo-instruction: inplace shorthand]
# ADDI	$s0, 10, $t2                  # $rd = i16 + $rs [warn: pseudo-instruction: reversed operands]
ADDI	$s0, $t1, -10                 # $rd = $rs + i16 <real instruction>
ADDI	$s0, -10                      # $rd = $rd + i16 [warn: pseudo-instruction: inplace shorthand]
# ADDI	$s0, -10, $t2                 # $rd = i16 + $rs [warn: pseudo-instruction: reversed operands]
ADDI	$s0, $t1, 2100000000          # $rd = $rs + i32 <real instruction>
ADDI	$s0, 2100000000               # $rd = $rd + i32 [warn: pseudo-instruction: inplace shorthand]
# ADDI	$s0, 2100000000, $t2          # $rd = i32 + $rs [warn: pseudo-instruction: reversed operands]
ADDI	$s0, $t1, -2100000000         # $rd = $rs + i32 <real instruction>
ADDI	$s0, -2100000000              # $rd = $rd + i32 [warn: pseudo-instruction: inplace shorthand
# ADDI	$s0, -2100000000, $t2         # $rd = i32 + $rs [warn: pseudo-instruction: reversed operands]
ADDI	$s0, $t1, 4000000000          # $rd = $rs + u32 <real instruction>
ADDI	$s0, 4000000000               # $rd = $rd + u32 [warn: pseudo-instruction: inplace shorthand]
# ADDI	$s0, 4000000000, $t2          # $rd = u32 + $rs [warn: pseudo-instruction: reversed operands]

### ADDIU ### - Add Immediate Unsigned Word - Arithmetic Instruction
# ADDIU	$s0, $t1, $t2                 # $rd = $rs + $rt [warn: pseudo-instruction: is an immidiate instruction]
# ADDIU	$s0, $t1                      # $rd = $rd + $rs [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: is an immidiate instruction]
ADDIU	$s0, $t1, 10                  # $rd = $rs + i16 <real instruction>
ADDIU	$s0, 10                       # $rd = $rd + i16 [warn: pseudo-instruction: inplace shorthand]
# ADDIU	$s0, 10, $t2                  # $rd = i16 + $rs [warn: pseudo-instruction: reversed operands]
ADDIU	$s0, $t1, -10                 # $rd = $rs + i16 <real instruction>
ADDIU	$s0, -10                      # $rd = $rd + i16 [warn: pseudo-instruction: inplace shorthand]
# ADDIU	$s0, -10, $t2                 # $rd = i16 + $rs [warn: pseudo-instruction: reversed operands]
ADDIU	$s0, $t1, 2100000000          # $rd = $rs + i32 [warn: pseudo-instruction: immidiate value too large for instruction]
ADDIU	$s0, 2100000000               # $rd = $rd + i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: immidiate value too large for instruction]
# ADDIU	$s0, 2100000000, $t2          # $rd = i32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: immidiate value too large for instruction]
ADDIU	$s0, $t1, -2100000000         # $rd = $rs + i32 [warn: pseudo-instruction: immidiate value too large for instruction]
ADDIU	$s0, -2100000000              # $rd = $rd + i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: immidiate value too large for instruction]
# ADDIU	$s0, -2100000000, $t2         # $rd = i32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: immidiate value too large for instruction]
ADDIU	$s0, $t1, 4000000000          # $rd = $rs + u32 [warn: pseudo-instruction: immidiate value too large for instruction]
ADDIU	$s0, 4000000000               # $rd = $rd + u32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: immidiate value too large for instruction]
# ADDIU	$s0, 4000000000, $t2          # $rd = u32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: immidiate value too large for instruction]

### ADDU ### - Add Unsigned Word - Arithmetic Instruction
ADDU	$s0, $t1, $t2                 # $rd = $rs + $rt <real instruction>
ADDU	$s0, $t1                      # $rd = $rd + $rs [warn: pseudo-instruction: inplace shorthand]
ADDU	$s0, $t1, 10                  # $rd = $rs + i16 [warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, 10                       # $rd = $rd + i16 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, 10, $t2                  # $rd = i16 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, $t1, -10                 # $rd = $rs + i16 [warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, -10                      # $rd = $rd + i16 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, -10, $t2                 # $rd = i16 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, $t1, 2100000000          # $rd = $rs + i32 [warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, 2100000000               # $rd = $rd + i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, 2100000000, $t2          # $rd = i32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, $t1, -2100000000         # $rd = $rs + i32 [warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, -2100000000              # $rd = $rd + i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, -2100000000, $t2         # $rd = i32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, $t1, 4000000000          # $rd = $rs + u32 [warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, 4000000000               # $rd = $rd + u32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
ADDU	$s0, 4000000000, $t2          # $rd = u32 + $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]

## AND ### - Bitwise And - Logical Instructions
AND	$s0, $t1, $t2                 # $rd = $rs & $rt <real instruction>
AND	$s0, $t1                      # $rd = $rd & $rs [warn: pseudo-instruction: inplace shorthand]
AND	$s0, $t1, 10                  # $rd = $rs & i16 [warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, 10                       # $rd = $rd & i16 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, 10, $t2                  # $rd = i16 & $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, $t1, -10                 # $rd = $rs & i16 [warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, -10                      # $rd = $rd & i16 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, -10, $t2                 # $rd = i16 & $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, $t1, 2100000000          # $rd = $rs & i32 [warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, 2100000000               # $rd = $rd & i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, 2100000000, $t2          # $rd = i32 & $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, $t1, -2100000000         # $rd = $rs & i32 [warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, -2100000000              # $rd = $rd & i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, -2100000000, $t2         # $rd = i32 & $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, $t1, 4000000000          # $rd = $rs & u32 [warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, 4000000000               # $rd = $rd & u32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: not an immidiate instruction]
AND	$s0, 4000000000, $t2          # $rd = u32 & $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: not an immidiate instruction]

### ANDI ### - Bitwise And Immediate - Logical Instructions
# ANDI	$s0, $t1, $t2                 # $rd = $rs & $rt [warn: pseudo-instruction: is an immidiate instruction]
# ANDI	$s0, $t1                      # $rd = $rd & $rs [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: is an immidiate instruction]
ANDI	$s0, $t1, 10                  # $rd = $rs & i16 <real instruction>
ANDI	$s0, 10                       # $rd = $rd & i16 [warn: pseudo-instruction: inplace shorthand]
# ANDI	$s0, 10, $t2                  # $rd = i16 & $rs [warn: pseudo-instruction: reversed operands]
ANDI	$s0, $t1, -10                 # $rd = $rs & i16 <real instruction>
# ANDI	$s0, -10                      # $rd = $rd & i16 [warn: pseudo-instruction: inplace shorthand]
# ANDI	$s0, -10, $t2                 # $rd = i16 & $rs [warn: pseudo-instruction: reversed operands]
ANDI	$s0, $t1, 2100000000          # $rd = $rs & i32 [warn: pseudo-instruction: immidiate value too large for instruction]
# ANDI	$s0, 2100000000               # $rd = $rd & i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: immidiate value too large for instruction]
# ANDI	$s0, 2100000000, $t2          # $rd = i32 & $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: immidiate value too large for instruction]
ANDI	$s0, $t1, -2100000000         # $rd = $rs & i32 [warn: pseudo-instruction: immidiate value too large for instruction]
# ANDI	$s0, -2100000000              # $rd = $rd & i32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: immidiate value too large for instruction]
# ANDI	$s0, -2100000000, $t2         # $rd = i32 & $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: immidiate value too large for instruction]
ANDI	$s0, $t1, 4000000000          # $rd = $rs & u32 [warn: pseudo-instruction: immidiate value too large for instruction]
# ANDI	$s0, 4000000000               # $rd = $rd & u32 [warn: pseudo-instruction: inplace shorthand; warn: pseudo-instruction: immidiate value too large for instruction]
# ANDI	$s0, 4000000000, $t2          # $rd = u32 & $rs [warn: pseudo-instruction: reversed operands; warn: pseudo-instruction: immidiate value too large for instruction]

### B ### Unconditional Branch (Assembly Idiom: AKA. a required Psuedo-Instruction) - Branch and Jump Instructions
B	main        # offset(i16) <real instruction>
B	100         # i16         <real instruction>
B	-100        # i16         <real instruction>

### BAL ### Branch and Link (Assembly Idiom: AKA. a required Psuedo-Instruction) - Branch and Jump Instructions
BAL	main        # offset(i16) <real instruction>
BAL	100         # i16         <real instruction>
BAL	-100        # i16         <real instruction>

### BEQ ### Branch on Equal - Branch and Jump Instructions

### BEQAL ### Branch on Equal and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BEQZ ### Branch on Equal to Zero - Branch and Jump Instructions [Psuedo-Instruction]

### BEQZAL ### Branch on Equal to Zero and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BGE ### Branch on Greater Than or Equal to - Branch and Jump Instructions [Psuedo-Instruction]

### BGEAL ### Branch on Greater Than or Equal to and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BGEZ ### Branch on Greater Than or Equal to Zero - Branch and Jump Instructions

### BGEZAL ### Branch on Greater Than or Equal to Zero and Link - Branch and Jump Instructions

### BGEU ### Branch on Unsigned Greater Than or Equal to - Branch and Jump Instructions [Psuedo-Instruction]

### BGEUAL ### Branch on Unsigned Greater Than or Equal to and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BGT ### Branch on Greater Than - Branch and Jump Instructions [Psuedo-Instruction]

### BGTAL ### Branch on Greater Than and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BGTZ ### Branch on Greater Than Zero - Branch and Jump Instructions

### BGTZAL ### Branch on Greater Than Zero and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BGTU ### Branch on Unsigned Greater Than - Branch and Jump Instructions [Psuedo-Instruction]

### BGTUAL ### Branch on Unsigned Greater Than and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BLE ### Branch on Less Than or Equal to - Branch and Jump Instructions [Psuedo-Instruction]

### BLEAL ### Branch on Less Than or Equal to and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BLEZ ### Branch on Less Than or Equal to Zero - Branch and Jump Instructions

### BLEZAL ### Branch on Less Than or Equal to Zero and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BLEU ### Branch on Unsigned Less Than or Equal to - Branch and Jump Instructions [Psuedo-Instruction]

### BLEUAL ### Branch on Unsigned Less Than or Equal to and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BLT ### Branch on Less Than - Branch and Jump Instructions [Psuedo-Instruction]

### BLTAL ### Branch on Less Than and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BLTZ ### Branch on Less Than Zero - Branch and Jump Instructions

### BLTZAL ### Branch on Less Than Zero and Link - Branch and Jump Instructions

### BLTU ### Branch on Unsigned Less Than - Branch and Jump Instructions [Psuedo-Instruction]

### BLTUAL ### Branch on Unsigned Less Than and Link- Branch and Jump Instructions [Psuedo-Instruction]

### BNE ### Branch on Not Equal - Branch and Jump Instructions

### BNEAL ### Branch on Not Equal and Link - Branch and Jump Instructions [Psuedo-Instruction]

### BNEZ ### Branch on Not Equal Zero - Branch and Jump Instructions [Psuedo-Instruction]

### BNEZAL ### Branch on Not Equal Zero and Link - Branch and Jump Instructions [Psuedo-Instruction]


BREAK # Breakpoint
CLO # Count Leading Ones in Word
CLZ # Count Leading Zeros in Word
DIV # Divide Word
DIVU # Divide Unsigned Word
EXT # Extract Bit Field
INS # Insert Bit Field
J # Jump
JAL # Jump and Link
JALR # Jump and Link Register
JALX # Jump and Link Exchange
JR # Jump Register
LB # Load Byte
LBU # Load Byte Unsigned
LH # Load Halfword
LHU # Load Halfword Unsigned
LUI # Load Upper Immediate
LW # Load Word
LWL # Load Word Left
LWR # Load Word Right
MADD # Multiply and Add Word to Hi, Lo
MADDU # Multiply and Add Unsigned Word to Hi, Lo
MFHI # Move From Hi Register
MFLO # Move From Lo Register
MOVF # Move Conditional on Floating Point False
MOVN # Move Conditional on Not Zero
MOVT # Move Conditional on Floating Point True
MOVZ # Move Conditional on Zero
MSUB # Multiply and Subtract Word to Hi, Lo
MSUBU # Multiply and Subtract Unsigned Word to Hi, Lo
MTHI # Move To Hi Register
MTLO # Move To Lo Register
MUL # Multiply Word to GPR
MULT # Multiply Word
MULTU # Multiply Unsigned Word
NOP # No Operation (Assembly Idiom: AKA. a required Psuedo-Instruction)
NOR # Bitwise Not Or
OR # Bitwise Or
ORI # Bitwise Or Immediate
ROTR # Rotate Word Right
ROTRV # Rotate Word Right Variable
SB # Store Byte
SEB # Sign-Extend Byte
SEH # Sign-Extend Halftword
SH # Store Halfword
SLL # Shift Word Left Logical
SLLV  # Shift Word Left Logical Variable
SLT # Set on Less Than
SLTI # Set on Less Than Immediate
SLTIU # Set on Less Than Immediate Unsigned
SLTU # Set on Less Than Unsigned
SRA # Shift Word Right Arithmetic
SRAV # Shift Word Right Arithmetic Variable
SRL # Shift Word Right Logical
SRLV # Shift Word Right Logical Variable
SUB # Subtract Word
SUBU # Subtract Unsigned Word
SW # Store Word
SWL # Store Word Left
SWR # Store Word Right
SYSCALL # System Call
TEQ # Trap if Equal
TEQI # Trap if Equal Immediate
TGE # Trap if Greater or Equal
TGEI # Trap if Greater or Equal Immediate
TGEIU # Trap if Greater or Equal Immediate Unsigned
TGEU # Trap if Greater or Equal Unsigned
TLT # Trap if Less Than
TLTI # Trap if Less Than Immediate
TLTIU # Trap if Less Than Immediate Unsigned
TLTU # Trap if Less Than Unsigned
TNE # Trap if Not Equal
TNEI # Trap if Not Equal Immediate
WSBH # Word Swap Bytes Within Halfwords
XOR # Bitwise Exclusive Or
XORI # Bitwise Exclusive Or Immediate

# MIPS32 - Not Implemented Release 5 Instructions:

# ABS_D     # Floating Point Absolute Value (Not Implemented - no FPU)
# ABS_PS    # Floating Point Absolute Value (Not Implemented - no FPU)
# ABS_S     # Floating Point Absolute Value (Not Implemented - no FPU)
# ADD_D     # Floating Point Add (Not Implemented - no FPU)
# ADD_PS    # Floating Point Add (Not Implemented - no FPU)
# ADD_S     # Floating Point Add (Not Implemented - no FPU)
# ALNV_PS   # Floating Point Align Variable (Not Implemented - no FPU)
# BC1F      # Branch on FP False (Not Implemented - no FPU)
# BC1FL     # Branch on FP False Likely (Not Implemented - no FPU)
# BC1T      # Branch on FP True (Not Implemented - no FPU)
# BC1TL     # Branch on FP True Likely (Not Implemented - no FPU)
# BC2F      # Branch on COP2 False (Not Implemented - no COP2)
# BC2FL     # Branch on COP2 False Likely (Not Implemented - no COP2)
# BC2T      # Branch on COP2 True (Not Implemented - no COP2)
# BC2TL     # Branch on COP2 True Likely (Not Implemented - no COP2)
# BEQL      # Branch on Equal Likely (Not Implemented - no `likely` instructions)
# BGEZALL   # Branch on Greater Than or Equal to Zero and Link Likely (Not Implemented - no `likely` instructions)
# BGEZL     # Branch on Greater Than or Equal to Zero Likely (Not Implemented - no `likely` instructions)
# BGTZL     # Branch on Greater Than Zero Likely (Not Implemented - no `likely` instructions)
# BLEZL     # Branch on Less Than or Equal to Zero Likely (Not Implemented - no `likely` instructions)
# BLTZALL   # Branch on Less Than Zero and Link Likely (Not Implemented - no `likely` instructions)
# BLTZL     # Branch on Less Than Zero Likely (Not Implemented - no `likely` instructions)
# BNEL      # Branch on Not Equal Likely (Not Implemented - no `likely` instructions)
# C_EQ_D    # Floating Point Compare Equal (Not Implemented - no FPU)
# C_EQ_PS   # Floating Point Compare Equal (Not Implemented - no FPU)
# C_EQ_S    # Floating Point Compare Equal (Not Implemented - no FPU)
# C_F_D     # Floating Point Compare False (Not Implemented - no FPU)
# C_F_PS    # Floating Point Compare False (Not Implemented - no FPU)
# C_F_S     # Floating Point Compare False (Not Implemented - no FPU)
# C_LE_D    # Floating Point Compare Less Than or Equal (Not Implemented - no FPU)
# C_LE_PS   # Floating Point Compare Less Than or Equal (Not Implemented - no FPU)
# C_LE_S    # Floating Point Compare Less Than or Equal (Not Implemented - no FPU)
# C_LT_D    # Floating Point Compare Less Than (Not Implemented - no FPU)
# C_LT_PS   # Floating Point Compare Less Than (Not Implemented - no FPU)
# C_LT_S    # Floating Point Compare Less Than (Not Implemented - no FPU)
# C_NGE_D   # Floating Point Compare Not Greater Than or Equal (Not Implemented - no FPU)
# C_NGE_PS  # Floating Point Compare Not Greater Than or Equal (Not Implemented - no FPU)
# C_NGE_S   # Floating Point Compare Not Greater Than or Equal (Not Implemented - no FPU)
# C_NGL_D   # Floating Point Compare Not Greater or Less Than (Not Implemented - no FPU)
# C_NGL_PS  # Floating Point Compare Not Greater or Less Than (Not Implemented - no FPU)
# C_NGL_S   # Floating Point Compare Not Greater or Less Than (Not Implemented - no FPU)
# C_NGLE_D  # Floating Point Compare Not Greater Than or Less Than or Equal (Not Implemented - no FPU)
# C_NGLE_PS # Floating Point Compare Not Greater Than or Less Than or Equal (Not Implemented - no FPU)
# C_NGLE_S  # Floating Point Compare Not Greater Than or Less Than or Equal (Not Implemented - no FPU)
# C_NGT_D   # Floating Point Compare Not Greater Than (Not Implemented - no FPU)
# C_NGT_PS  # Floating Point Compare Not Greater Than (Not Implemented - no FPU)
# C_NGT_S   # Floating Point Compare Not Greater Than (Not Implemented - no FPU)
# C_OLE_D   # Floating Point Compare Ordered and Less Than or Equal (Not Implemented - no FPU)
# C_OLE_PS  # Floating Point Compare Ordered and Less Than or Equal (Not Implemented - no FPU)
# C_OLE_S   # Floating Point Compare Ordered and Less Than or Equal (Not Implemented - no FPU)
# C_OLT_D   # Floating Point Compare Ordered and Less Than (Not Implemented - no FPU)
# C_OLT_PS  # Floating Point Compare Ordered and Less Than (Not Implemented - no FPU)
# C_OLT_S   # Floating Point Compare Ordered and Less Than (Not Implemented - no FPU)
# C_SEQ_D   # Floating Point Compare Signaling Equal (Not Implemented - no FPU)
# C_SEQ_PS  # Floating Point Compare Signaling Equal (Not Implemented - no FPU)
# C_SEQ_S   # Floating Point Compare Signaling Equal (Not Implemented - no FPU)
# C_SF_D    # Floating Point Compare Signaling False (Not Implemented - no FPU)
# C_SF_PS   # Floating Point Compare Signaling False (Not Implemented - no FPU)
# C_SF_S    # Floating Point Compare Signaling False (Not Implemented - no FPU)
# C_UEQ_D   # Floating Point Compare Unordered or Equal (Not Implemented - no FPU)
# C_UEQ_PS  # Floating Point Compare Unordered or Equal (Not Implemented - no FPU)
# C_UEQ_S   # Floating Point Compare Unordered or Equal (Not Implemented - no FPU)
# C_ULE_D   # Floating Point Compare Unordered or Less Than or Equal (Not Implemented - no FPU)
# C_ULE_PS  # Floating Point Compare Unordered or Less Than or Equal (Not Implemented - no FPU)
# C_ULE_S   # Floating Point Compare Unordered or Less Than or Equal (Not Implemented - no FPU)
# C_ULT_D   # Floating Point Compare Unordered or Less Than (Not Implemented - no FPU)
# C_ULT_PS  # Floating Point Compare Unordered or Less Than (Not Implemented - no FPU)
# C_ULT_S   # Floating Point Compare Unordered or Less Than (Not Implemented - no FPU)
# C_UN_D    # Floating Point Compare Unordered (Not Implemented - no FPU)
# C_UN_PS   # Floating Point Compare Unordered (Not Implemented - no FPU)
# C_UN_S    # Floating Point Compare Unordered (Not Implemented - no FPU)
# CACHE     # Perform Cache Operation
# CACHEE    # Perform Cache Operation EVA (Not Implemented - no virtual memory)
# CEIL_L_D  # Floating Point Ceiling Convert to Long Fixed Point (Not Implemented - no FPU)
# CEIL_L_S  # Floating Point Ceiling Convert to Long Fixed Point (Not Implemented - no FPU)
# CEIL_W_D  # Floating Point Ceiling Convert to Word Fixed Point (Not Implemented - no FPU)
# CEIL_W_S  # Floating Point Ceiling Convert to Word Fixed Point (Not Implemented - no FPU)
# CFC1      # Move Control Word from Floating Point
# CFC2      # Move Control Word from Coprocessor 2
# COP2      # Coprocessor Operation to Coprocessor 2 (Not Implemented - no FPU)
# CTC1      # Move Control Word to Floating Point
# CTC2      # Move Control Word to Coprocessor 2
# CVT_D_L   # Floating Point Convert to Double Floating Point (Not Implemented - no FPU)
# CVT_D_S   # Floating Point Convert to Double Floating Point (Not Implemented - no FPU)
# CVT_D_W   # Floating Point Convert to Double Floating Point (Not Implemented - no FPU)
# CVT_L_D   # Floating Point Convert to Long Fixed Point (Not Implemented - no FPU)
# CVT_L_S   # Floating Point Convert to Long Fixed Point (Not Implemented - no FPU)
# CVT_PS_S  # Floating Point Convert to Paired Single Floating Point (Not Implemented - no FPU)
# CVT_S_D   # Floating Point Convert to Single Floating Point (Not Implemented - no FPU)
# CVT_S_L   # Floating Point Convert to Single Floating Point (Not Implemented - no FPU)
# CVT_S_PL  # Floating Point Convert to Single Floating Point (Not Implemented - no FPU)
# CVT_S_PU  # Floating Point Convert to Single Floating Point (Not Implemented - no FPU)
# CVT_S_W   # Floating Point Convert to Single Floating Point (Not Implemented - no FPU)
# CVT_W_D   # Floating Point Convert to Word Fixed Point (Not Implemented - no FPU)
# CVT_W_S   # Floating Point Convert to Word Fixed Point (Not Implemented - no FPU)
# DERET     # Debug Exception Return (Not Implemented - no exceptions)
# DI        # Disable Interrupts (Not Implemented - no interrupts)
# DIV_D     # Floating Point Divide (Not Implemented - no FPU)
# DIV_S     # Floating Point Divide (Not Implemented - no FPU)
# EHB       # Execution Hazard Barrier (Not Implemented - no hazard barrier)
# EI        # Enable Interrupts (Not Implemented - no interrupts)
# ERET      # Exception Return (Not Implemented - no exceptions)
# FLOOR_L_D # Floating Point Floor Convert to Long Fixed Point (Not Implemented - no FPU)
# FLOOR_L_S # Floating Point Floor Convert to Long Fixed Point (Not Implemented - no FPU)
# FLOOR_W_D # Floating Point Floor Convert to Word Fixed Point (Not Implemented - no FPU)
# FLOOR_W_S # Floating Point Floor Convert to Word Fixed Point (Not Implemented - no FPU)
# JALR.HB   # Jump and Link Register with Hazard Barrier (Not Implemented - no hazard barrier)
# JR_HB     # Jump Register with Hazard Barrier (Not Implemented - no hazard barrier)
# LBE       # Load Byte EVA (Not Implemented - no virtual addressing)
# LBUE      # Load Byte Unsigned EVA (Not Implemented - no virtual addressing)
# LDC1      # Load Doubleword to Floating Point (Not Implemented - no FPU)
# LDC2      # Load Doubleword to Coprocessor 2
# LDXC1     # Load Doubleword Indexed to Floating Point (Not Implemented - no FPU)
# LHE       # Load Halfword EVA (Not Implemented - no virtual addressing)
# LHUE      # Load Halfword Unsigned EVA (Not Implemented - no virtual addressing)
# LL        # Load Linked Word (Not Implemented - no atomic memory operations)
# LLE       # Load Linked Word EVA (Not Implemented - no atomic memory operations)
# LUXC1     # Load Doubleword Indexed Unaligned to Floating Point (Not Implemented - no FPU)
# LWE       # Load Word EVA (Not Implemented - no virtual addressing)
# LWC1      # Load Word to Floating Point (Not Implemented - no FPU)
# LWC2      # Load Word to Coprocessor 2
# LWLE      # Load Word Left EVA (Not Implemented - no virtual addressing)
# LWRE      # Load Word Right EVA (Not Implemented - no virtual addressing)
# LWXC1     # Load Word Indexed to Floating Point (Not Implemented - no FPU)
# MADD_D    # Floating Point Multiply Add (Not Implemented - no FPU)
# MADD_PS   # Floating Point Multiply Add (Not Implemented - no FPU)
# MADD_S    # Floating Point Multiply Add (Not Implemented - no FPU)
# MFC1      # Move Word from Floating Point
# MFC2      # Move Word from Coprocessor 2
# MFHC1     # Move Word from High Half of Floating Point Register
# MFHC2     # Move Word from High Half of Coprocessor 2 Register
# MOV_D     # Floating Point Move (Not Implemented - no floating point)
# MOV_PS    # Floating Point Move (Not Implemented - no floating point)
# MOV_S     # Floating Point Move (Not Implemented - no floating point)
# MOVF_D    # Floating Point Move Conditional on Floating Point False (Not Implemented - no floating point)
# MOVF_PS   # Floating Point Move Conditional on Floating Point False (Not Implemented - no floating point)
# MOVF_S    # Floating Point Move Conditional on Floating Point False (Not Implemented - no floating point)
# MOVN_D    # Floating Point Move Conditional on Not Zero (Not Implemented - no floating point)
# MOVN_PS   # Floating Point Move Conditional on Not Zero (Not Implemented - no floating point)
# MOVN_S    # Floating Point Move Conditional on Not Zero (Not Implemented - no floating point)
# MOVT_D    # Floating Point Move Conditional on Floating Point True (Not Implemented - no floating point)
# MOVT_PS   # Floating Point Move Conditional on Floating Point True (Not Implemented - no floating point)
# MOVT_S    # Floating Point Move Conditional on Floating Point True (Not Implemented - no floating point)
# MOVZ_D    # Floating Point Move Conditional on Zero (Not Implemented - no floating point)
# MOVZ_PS   # Floating Point Move Conditional on Zero (Not Implemented - no floating point)
# MOVZ_S    # Floating Point Move Conditional on Zero (Not Implemented - no floating point)
# MSUB_D    # Floating Point Multiply Subtract (Not Implemented - no FPU)
# MSUB_PS   # Floating Point Multiply Subtract (Not Implemented - no FPU)
# MSUB_S    # Floating Point Multiply Subtract (Not Implemented - no FPU)
# MTC1      # Move Word to Floating Point
# MTC2      # Move Word to Coprocessor 2
# MTHC1     # Move Word to High Half of Floating Point Register
# MTHC2     # Move Word to High Half of Coprocessor 2 Register
# MUL_D     # Floating Point Multiply (Not Implemented - no FPU)
# MUL_PS    # Floating Point Multiply (Not Implemented - no FPU)
# MUL_S     # Floating Point Multiply (Not Implemented - no FPU)
# NEG_D     # Floating Point Negate (Not Implemented - no FPU)
# NEG_PS    # Floating Point Negate (Not Implemented - no FPU)
# NEG_S     # Floating Point Negate (Not Implemented - no FPU)
# NMADD_D   # Floating Point Negative Multiply Add (Not Implemented - no FPU)
# NMADD_PS  # Floating Point Negative Multiply Add (Not Implemented - no FPU)
# NMADD_S   # Floating Point Negative Multiply Add (Not Implemented - no FPU)
# NMSUB_D   # Floating Point Negative Multiply Subtract (Not Implemented - no FPU)
# NMSUB_PS  # Floating Point Negative Multiply Subtract (Not Implemented - no FPU)
# NMSUB_S   # Floating Point Negative Multiply Subtract (Not Implemented - no FPU)
# PAUSE     # Wait for LLBit to Clear (Not Implemented - no LLBit)
# PLL_PS    # Floating Point Pair Lower Lower (Not Implemented - no FPU)
# PLU_PS    # Floating Point Pair Lower Upper (Not Implemented - no FPU)
# PREF      # Prefetch (Not Implemented - no virtual addressing)
# PREFE     # Prefetch EVA (Not Implemented - no virtual addressing)
# PREFX     # Prefetch Indexed (Not Implemented - no virtual addressing)
# PUL_PS    # Floating Point Pair Upper Lower (Not Implemented - no FPU)
# PUU_PS    # Floating Point Pair Upper Upper (Not Implemented - no FPU)
# RDHWR     # Read Hardware Register (Not Implemented - no hardware registers)
# RDPGPR    # Read GPR from Previous Shadow Set (Not Implemented - no shadow set)
# RECIP_D   # Floating Point Reciprocal (Not Implemented - no FPU)
# RECIP_S   # Floating Point Reciprocal (Not Implemented - no FPU)
# ROUND_L_D # Floating Point Round to Long (Not Implemented - no FPU)
# ROUND_L_S # Floating Point Round to Long (Not Implemented - no FPU)
# ROUND_W_D # Floating Point Round to Word (Not Implemented - no FPU)
# ROUND_W_S # Floating Point Round to Word (Not Implemented - no FPU)
# RSQRT_D   # Floating Point Reciprocal Square Root (Not Implemented - no FPU)
# RSQRT_S   # Floating Point Reciprocal Square Root (Not Implemented - no FPU)
# SBE       # Store Byte EVA (Not Implemented - no virtual addressing)
# SC        # Store Conditional Word (Not Implemented - no atomic memory operations)
# SCE       # Store Conditional Word EVA (Not Implemented - no atomic memory operations)
# SDBBP     # Software Debug Breakpoint (Not Implemented - no exceptions)
# SDC1      # Store Doubleword from Floating Point (Not Implemented - no FPU)
# SDC2      # Store Doubleword from Coprocessor 2 (Not Implemented - no coprocessor 2)
# SDXC1     # Store Doubleword Indexed from Floating Point (Not Implemented - no FPU)
# SHE       # Store Halfword EVA (Not Implemented - no virtual addressing)
# SQRT_D    # Floating Point Square Root (Not Implemented - no FPU)
# SQRT_S    # Floating Point Square Root (Not Implemented - no FPU)
# SSNOP     # Superscalar No Operation (Not Implemented - no superscalar)
# SUB_D     # Floating Point Subtract (Not Implemented - no FPU)
# SUB_PS    # Floating Point Subtract (Not Implemented - no FPU)
# SUB_S     # Floating Point Subtract (Not Implemented - no FPU)
# SUXC1     # Store Doubleword Indexed Unaligned from Floating Point (Not Implemented - no FPU)
# SWE       # Store Word EVA (Not Implemented - no virtual addressing)
# SWC1      # Store Word from Floating Point (Not Implemented - no FPU)
# SWC2      # Store Word from COP2 (Not Implemented - no COP2)
# SWLE      # Store Word Left EVA (Not Implemented - no virtual addressing)
# SWRE      # Store Word Right EVA (Not Implemented - no virtual addressing)
# SWXC1     # Store Word Indexed from Floating Point (Not Implemented - no FPU)
# SYNC      # Synchronize Shared Memory (Not Implemented - no shared memory)
# SYNCI     # Synchronize Caches to Make Instruction Writes Effective (Not Implemented - no caches)
# TLBP      # Probe TLB for Matching Entry (Not Implemented - no virtual addressing)
# TLBR      # Read Indexed TLB Entry (Not Implemented - no virtual addressing)
# TLBWI     # Write Indexed TLB Entry (Not Implemented - no virtual addressing)
# TLBWR     # Write Random TLB Entry (Not Implemented - no virtual addressing)
# TRUNC_L_D # Floating Point Truncate to Long (Not Implemented - no FPU)
# TRUNC_L_S # Floating Point Truncate to Long (Not Implemented - no FPU)
# TRUNC_W_D # Floating Point Truncate to Word (Not Implemented - no FPU)
# TRUNC_W_S # Floating Point Truncate to Word (Not Implemented - no FPU)
# WAIT      # Enter Standby Mode (Not Implemented - no standby mode)
# WRPGPR    # Write GPR to Previous Shadow Set (Not Implemented - no shadow sets)

# SPIM Pseudo-Instructions:

LA
LD
L_D
L_S
LI_D
LI
LI_S
MFC1_D
MOVE
MTC1_D
MULO
MULOU
NEG
NEGU
NOT
REM
REMU
ROL
ROR
S_D
S_S
SD
SEQ
SGE
SGEU
SGT
SGTU
SLE
SLEU
SNE
ULH
ULHU
ULW
USH
USW
