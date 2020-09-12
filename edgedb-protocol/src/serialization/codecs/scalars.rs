use std::str;
use std::mem::size_of;
use std::convert::{TryFrom, TryInto};
use std::time::SystemTime;

use bytes::{Buf, BufMut};

use crate::errors::{self, DecodeError};
use crate::model::{Json, Uuid};
use snafu::{ResultExt, OptionExt, ensure};
use crate::model::{Duration, LocalDate, LocalTime, LocalDatetime, Datetime, BigInt, Decimal};
use super::{Codec, Input, Output};
use crate::queryable::{DescriptorContext, DescriptorMismatch};
use crate::descriptors::TypePos;
use crate::serialization::type_ids;

#[derive(Debug, Default)]
pub struct ScalarCodec;

fn check_scalar(ctx: &DescriptorContext, type_pos: TypePos, type_id: Uuid, name: &str) -> Result<(), DescriptorMismatch> {
    use crate::descriptors::Descriptor::{Scalar, BaseScalar};
    let desc = ctx.get(type_pos)?;
    match desc {
        Scalar(scalar) => {
            return check_scalar(ctx, scalar.base_type_pos, type_id, name);
        }
        BaseScalar(base) if base.id == type_id => {
            return Ok(());
        }
        _ => {}
    }
    Err(ctx.wrong_type(desc, name))
}

fn ensure_exact_size(buf:&[u8], expected_size: usize) -> Result<(), DecodeError> {
    if buf.len() != expected_size {
        if buf.len() < expected_size {
            return errors::Underflow.fail();
        } else {
            return errors::ExtraData.fail();
        }
    }
    Ok(())
}

fn bytes_sized<'t>(input:Input<'t>, expected_size: usize) -> Result<&'t [u8], DecodeError> {
    let bytes = input.bytes()?;
    ensure_exact_size(bytes, expected_size)?;
    Ok(bytes)
}

impl<'t> Codec<'t, String> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<String, DecodeError> {
        let str :&str = ScalarCodec::default().decode(input)?;
        Ok(str.to_owned())
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_STR, "str")
    }
    fn encode(&self, output: &mut Output, val: &String) -> Result<(), errors::EncodeError> {
        ScalarCodec::default().encode(output, &val.as_str())
    }
}

impl<'t> Codec<'t, &'t str> for ScalarCodec {
    fn decode(&self, input: Input<'t>) -> Result<&'t str, DecodeError> {
        let bytes: &'t [u8] = ScalarCodec::default().decode(input)?;
        str::from_utf8(bytes).context(errors::InvalidUtf8)
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_STR, "str")
    }
    fn encode(&self, output: &mut Output, val: &&'t str) -> Result<(), errors::EncodeError> {
        ScalarCodec::default().encode(output, &val.as_bytes())
    }
}

impl<'t> Codec<'t, Json> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<Json, DecodeError> {
        let mut buf = input.bytes()?;
        ensure!(buf.remaining() >= 1, errors::Underflow);
        let format = buf.get_u8();
        ensure!(format == 1, errors::InvalidJsonFormat);
        let val = str::from_utf8(buf)
            .context(errors::InvalidUtf8)?
            .to_owned();
        Ok(Json::new_unchecked(val))
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_JSON, "json")
    }
    fn encode(&self, output: &mut Output, val: &Json) -> Result<(), errors::EncodeError> {
        ScalarCodec::default().encode(output, &val.as_bytes())       
    }
}

impl<'t> Codec<'t, Uuid> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<Uuid, DecodeError> {
        let buf = bytes_sized(input, 16)?;
        let uuid = Uuid::from_slice(buf).unwrap();
        Ok(uuid)
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_UUID, "uuid")
    }
    fn encode(&self, buf: &mut Output, val: &Uuid) -> Result<(), errors::EncodeError> {
        buf.reserve(16);
        buf.extend(val.as_bytes());
        Ok(())
    }
}

impl<'t> Codec<'t, bool> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<bool, DecodeError> {
        let buf = bytes_sized(input, 1)?;
        let res = match buf[0] {
            0x00 => false,
            0x01 => true,
            _ => errors::InvalidBool.fail()?,
        };
        Ok(res)
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_BOOL, "bool")
    }
    fn encode(&self, buf: &mut Output, val: &bool) -> Result<(), errors::EncodeError> {
        buf.reserve(1);
        buf.put_u8(match val {
            true => 1,
            false => 0,
        });
        Ok(())   
    }  
}

impl<'t> Codec<'t, i16> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<i16, DecodeError> {
        let mut buf = bytes_sized(input, size_of::<i16>())?;
        return Ok(buf.get_i16());
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_INT16, "int16")
    }
    fn encode(&self, buf: &mut Output, val: &i16) -> Result<(), errors::EncodeError> {
        buf.reserve(size_of::<i16>());
        buf.put_i16(*val);
        Ok(())
    }
}

impl<'t> Codec<'t, i32> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<i32, DecodeError> {
        let mut buf = bytes_sized(input, size_of::<i32>())?;
        return Ok(buf.get_i32());
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_INT32, "int32")
    }
    fn encode(&self, buf: &mut Output, val: &i32) -> Result<(), errors::EncodeError> {
        buf.reserve(size_of::<i32>());
        buf.put_i32(*val);
        Ok(())
    }
}

impl<'t> Codec<'t, i64> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<i64, DecodeError> {
        let mut buf = bytes_sized(input, size_of::<i64>())?;
        return Ok(buf.get_i64());
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_INT64, "int64")
    }
    fn encode(&self, buf: &mut Output, val: &i64) -> Result<(), errors::EncodeError> {
        buf.reserve(size_of::<i64>());
        buf.put_i64(*val);
        Ok(())
    }
}

impl<'t> Codec<'t, f32> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<f32, DecodeError> {
        let mut buf = bytes_sized(input, size_of::<f32>())?;
        return Ok(buf.get_f32());
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_FLOAT32, "float32")
    }
    fn encode(&self, buf: &mut Output, val: &f32) -> Result<(), errors::EncodeError> {
        buf.reserve(size_of::<f32>());
        buf.put_f32(*val);
        Ok(())
    }
}

impl<'t> Codec<'t, f64> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<f64, DecodeError> {
        let mut buf = bytes_sized(input, size_of::<f64>())?;
        return Ok(buf.get_f64());
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_FLOAT64, "float64")
    }
    fn encode(&self, buf: &mut Output, val: &f64) -> Result<(), errors::EncodeError> {
        buf.reserve(size_of::<f64>());
        buf.put_f64(*val);
        Ok(())
    }
}

impl<'t> Codec<'t, &'t [u8]> for ScalarCodec {
    fn decode(&self, input: Input<'t>) -> Result<&'t [u8], DecodeError> {
        input.bytes()
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_BYTES, "bytes")
    }
    fn encode(&self, buf: &mut Output, val: &&'t [u8]) -> Result<(), errors::EncodeError> {
        buf.extend(*val);
        Ok(())
    }    
}

impl<'t> Codec<'t, Vec<u8>> for ScalarCodec {
    fn decode(&self, input: Input<'t>) -> Result<Vec<u8>, DecodeError> {
        let bytes : &'t[u8] = ScalarCodec::default().decode(input)?;
        Ok(bytes.to_owned())
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_BYTES, "bytes")
    }
    fn encode(&self, output: &mut Output, val: &Vec<u8>) -> Result<(), errors::EncodeError> {
        ScalarCodec::default().encode(output, &val.as_slice())
    }
}

impl<'t> Codec<'t, Decimal> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<Decimal, DecodeError> {
        let mut buf = input.bytes()?;
        ensure!(buf.remaining() >= 8, errors::Underflow);
        let ndigits = buf.get_u16() as usize;
        let weight = buf.get_i16();
        let negative = match buf.get_u16() {
            0x0000 => false,
            0x4000 => true,
            _ => errors::BadSign.fail()?,
        };
        let decimal_digits = buf.get_u16();
        ensure_exact_size(buf, ndigits*2)?;
        let mut digits = Vec::with_capacity(ndigits);
        for _ in 0..ndigits {
            digits.push(buf.get_u16());
        }
        Ok(Decimal {
            negative, weight, decimal_digits, digits,
        })
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_DECIMAL, "decimal")
    }
    fn encode(&self, buf: &mut Output, val: &Decimal) -> Result<(), errors::EncodeError> {
        buf.reserve(8 + val.digits.len()*2);
        buf.put_u16(val.digits.len().try_into().ok()
                .context(errors::BigIntTooLong)?);
        buf.put_i16(val.weight);
        buf.put_u16(if val.negative { 0x4000 } else { 0x0000 });
        buf.put_u16(val.decimal_digits);
        for &dig in &val.digits {
            buf.put_u16(dig);
        }
        Ok(())
    }
}

impl<'t> Codec<'t, BigInt> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<BigInt, DecodeError> {
        let mut buf = input.bytes()?;
        ensure!(buf.remaining() >= 8, errors::Underflow);
        let ndigits = buf.get_u16() as usize;
        let weight = buf.get_i16();
        let negative = match buf.get_u16() {
            0x0000 => false,
            0x4000 => true,
            _ => errors::BadSign.fail()?,
        };
        let decimal_digits = buf.get_u16();
        ensure!(decimal_digits == 0, errors::NonZeroReservedBytes);
        let mut digits = Vec::with_capacity(ndigits);
        ensure_exact_size(buf, ndigits*2)?;
        for _ in 0..ndigits {
            digits.push(buf.get_u16());
        }
        Ok(BigInt {
            negative, weight, digits,
        })
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_BIGINT, "bigint")
    }
    fn encode(&self, buf: &mut Output, val: &BigInt) -> Result<(), errors::EncodeError> {
        buf.reserve(8 + val.digits.len()*2);
        buf.put_u16(val.digits.len().try_into().ok()
                .context(errors::BigIntTooLong)?);
        buf.put_i16(val.weight);
        buf.put_u16(if val.negative { 0x4000 } else { 0x0000 });
        buf.put_u16(0);
        for &dig in &val.digits {
            buf.put_u16(dig);
        };
        Ok(())
     }
}

impl<'t> Codec<'t, Duration> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<Duration, DecodeError> {
        let mut buf = bytes_sized(input, 16)?;
        let micros = buf.get_i64();
        let days = buf.get_u32();
        let months = buf.get_u32();
        ensure!(months == 0 && days == 0, errors::NonZeroReservedBytes);
        Ok(Duration::from_micros(micros))
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_DURATION, "duration")
    }
    fn encode(&self, buf: &mut Output, val: &Duration) -> Result<(), errors::EncodeError> {
        buf.reserve(16);
        buf.put_i64(val.to_micros());
        buf.put_u32(0);
        buf.put_u32(0);
        Ok(())
    }
}

impl<'t> Codec<'t, Datetime> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<Datetime, DecodeError> {
        let micros: i64 = ScalarCodec::default().decode(input)?;
        Ok(Datetime::from_micros(micros))
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_DATETIME, "datetime")
    }
    fn encode(&self, output: &mut Output, val: &Datetime) -> Result<(), errors::EncodeError> {
        ScalarCodec::default().encode(output, &val.to_micros())
    }
}

impl<'t> Codec<'t, SystemTime> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<SystemTime, DecodeError> {
        let datetime: Datetime = ScalarCodec::default().decode(input)?;
        Ok(datetime.into())
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::STD_DATETIME, "datetime")
    }
    fn encode(&self, output: &mut Output, val: &SystemTime) -> Result<(), errors::EncodeError> {
        let datetime = Datetime::try_from(*val).ok().context(errors::DatetimeRange)?;
        ScalarCodec::default().encode(output, &datetime)
    }
}

impl<'t> Codec<'t, LocalDatetime> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<LocalDatetime, DecodeError> {
        let micros : i64 = ScalarCodec::default().decode(input)?;
        Ok(LocalDatetime::from_micros(micros))
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::CAL_LOCAL_DATETIME, "cal::local_datetime")
    }
    fn encode(&self, output: &mut Output, val: &LocalDatetime) -> Result<(), errors::EncodeError> {
        ScalarCodec::default().encode(output, &val.to_micros())
    }
}

impl<'t> Codec<'t, LocalDate> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<LocalDate, DecodeError> {
        let days : i32 = ScalarCodec::default().decode(input)?;
        Ok(LocalDate::from_days(days))
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::CAL_LOCAL_DATE, "cal::local_date")
    }
    fn encode(&self, output: &mut Output, val: &LocalDate) -> Result<(), errors::EncodeError> {
        ScalarCodec::default().encode(output, &val.to_days())
    }
}

impl<'t> Codec<'t, LocalTime> for ScalarCodec {
    fn decode(&self, input: Input) -> Result<LocalTime, DecodeError> {
        let micros : i64 = ScalarCodec::default().decode(input)?;
        LocalTime::try_from_micros(micros as u64).ok().context(errors::InvalidDate)
    }
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        check_scalar(ctx, type_pos, type_ids::CAL_LOCAL_TIME, "cal::local_time")
    }
    fn encode(&self, output: &mut Output, val: &LocalTime) -> Result<(), errors::EncodeError> {
        ScalarCodec::default().encode(output, &(val.to_micros() as i64))
    }
}
