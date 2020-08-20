use uuid::Uuid;

pub const STD_UUID: Uuid = Uuid::from_u128(0x100);
pub const STD_STR: Uuid = Uuid::from_u128(0x101);
pub const STD_BYTES: Uuid = Uuid::from_u128(0x102);
pub const STD_INT16: Uuid = Uuid::from_u128(0x103);
pub const STD_INT32: Uuid = Uuid::from_u128(0x104);
pub const STD_INT64: Uuid = Uuid::from_u128(0x105);
pub const STD_FLOAT32: Uuid = Uuid::from_u128(0x106);
pub const STD_FLOAT64: Uuid = Uuid::from_u128(0x107);
pub const STD_DECIMAL: Uuid = Uuid::from_u128(0x108);
pub const STD_BOOL: Uuid = Uuid::from_u128(0x109);
pub const STD_DATETIME: Uuid = Uuid::from_u128(0x10a);
pub const CAL_LOCAL_DATETIME: Uuid = Uuid::from_u128(0x10b);
pub const CAL_LOCAL_DATE: Uuid = Uuid::from_u128(0x10c);
pub const CAL_LOCAL_TIME: Uuid = Uuid::from_u128(0x10d);
pub const STD_DURATION: Uuid = Uuid::from_u128(0x10e);
pub const STD_JSON: Uuid = Uuid::from_u128(0x10f);
pub const STD_BIGINT: Uuid = Uuid::from_u128(0x110);