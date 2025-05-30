const EXECUTE_BIT_FLAG: u16 = 1u16;
const WRITE_BIT_FLAG: u16 = 2u16;
const READ_BIT_FLAG: u16 = 4u16;

pub fn has_execute_perm(perm: u16) -> bool {
    return (perm & EXECUTE_BIT_FLAG) != 0;
}

pub fn has_write_perm(perm: u16) -> bool {
    return (perm & WRITE_BIT_FLAG) != 0;
}

pub fn has_read_perm(perm: u16) -> bool {
    return (perm & READ_BIT_FLAG) != 0;
}
