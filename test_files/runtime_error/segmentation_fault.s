main:
    b       null

null:
    lb      $t0, 0x00000000
    
below_text:
    lb      $t0, 0x00300000
    
above_heap:
    lb      $t0, 0x10040001
    
below_stack:
    lb      $t0, 0x70000000
    
above_stack:
    lb      $t0, 0x80000000
