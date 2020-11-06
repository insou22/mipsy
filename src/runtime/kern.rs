pub const fn get_kern_instns(entry_point: u32) -> [u32; 7] {
    let ep_upper = entry_point >> 16;
    let ep_lower = entry_point & 0xFFFF;
    
    let lui = 0x3c1a0000 | ep_upper;
    let ori = 0x375a0000 | ep_lower;
    [
        lui,        // la      $k0, main
        ori,        // .................
        0x34020000, // li      $v0, 0
        0x03400009, // jalr    $k0
        0x00022021, // move    $a0, $v0
        0x34020011, // li      $v0, 17
        0x0000000c, // syscall
    ]
}
