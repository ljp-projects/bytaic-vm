pub const PUSH: u8 = 0u8;
pub const PRINT: u8 = 1u8;
pub const NADD: u8 = 2u8;
pub const NSUB: u8 = 3u8;
pub const NMUL: u8 = 4u8;
pub const NDIV: u8 = 5u8;
pub const NMOD: u8 = 6u8;
pub const NPOW: u8 = 7u8;
pub const STORE: u8 = 8u8;
pub const LOAD: u8 = 9u8;
pub const SMUL: u8 = 10u8;
pub const RET: u8 = 11u8;
pub const CALL: u8 = 12u8;
pub const GET: u8 = 13u8;
pub const WRITE: u8 = 14u8;
pub const READLN: u8 = 15u8;
pub const MARKER: u8 = 16u8;
pub const GOTO: u8 = 17u8;

/// If a boolean on the stack is true, go to the marker at IDX.
/// 
/// GOTO_IF (18) IDX (u32)
pub const GOTO_IF: u8 = 18u8;

pub const EQ: u8 = 19u8;
pub const NGT: u8 = 20u8;
pub const NLT: u8 = 21u8;