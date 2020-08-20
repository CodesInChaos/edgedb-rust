use crate::queryable::{DescriptorContext, DescriptorMismatch};
use crate::errors::DecodeError;
use crate::descriptors::{Descriptor, TypePos};
use crate::serialization::decode_composite::DecodeArrayLike;
use std::iter::FromIterator;
use super::{Codec, Input};

#[derive(Debug)]
pub struct FromIteratorCodec<InnerCodec>(InnerCodec);

impl<InnerCodec> FromIteratorCodec<InnerCodec> {
    pub fn new(inner:InnerCodec) -> FromIteratorCodec<InnerCodec> {
        Self(inner)
    }
}

impl<'t, T:IntoIterator + FromIterator<<T as IntoIterator>::Item>, InnerCodec> Codec<'t, T> for FromIteratorCodec<InnerCodec>
    where InnerCodec: Codec<'t, <T as IntoIterator>::Item>
{
    fn decode(&self, buf: Input<'t>) -> Result<T, DecodeError> {
        let elements = DecodeArrayLike::new(buf)?;
        let elements = elements.map(|e| self.0.decode(e?));
        elements.collect::<Result<T, DecodeError>>()
    }

    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos)
        -> Result<(), DescriptorMismatch>
    {
        let desc = ctx.get(type_pos)?;
        let element_type_pos = match desc {
            Descriptor::Set(desc) => desc.type_pos,
            Descriptor::Array(desc) => desc.type_pos,
            _ => return Err(ctx.wrong_type(desc, "array or set"))
        };
        self.0.check_descriptor(ctx, element_type_pos)
    }
    fn encode(&self, output: &mut crate::serialization::Output, val: &T) -> Result<(), crate::errors::EncodeError> {
        todo!()
    } 
}