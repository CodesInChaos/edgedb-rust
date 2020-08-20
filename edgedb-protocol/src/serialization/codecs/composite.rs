use crate::queryable::{DescriptorContext, DescriptorMismatch};
use crate::errors::DecodeError;
use crate::descriptors::TypePos;
use super::{Codec, Input, FromIteratorCodec};

#[derive(Debug)]
pub struct CompositeCodec<InnerCodec>(InnerCodec);

impl<'t, T, InnerCodec> Codec<'t, Option<T>> for CompositeCodec<InnerCodec>
    where InnerCodec:Codec<'t, T>
{
    fn decode(&self, input: Input<'t>) -> Result<Option<T>, DecodeError> {
        match input.0 {
            Some(_) => { self.0.decode(input).map(|val| Some(val)) }
            None => { Ok(None) }
        }
    }

    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos)
        -> Result<(), DescriptorMismatch>
    {
        self.0.check_descriptor(ctx, type_pos)
    }
    fn encode(&self, output: &mut crate::serialization::Output, val: &Option<T>) -> Result<(), crate::errors::EncodeError> {
        todo!()
    }
}

impl<'t, T, InnerCodec> Codec<'t, Vec<T>> for CompositeCodec<InnerCodec>
    where InnerCodec:Codec<'t, T>
{
    fn decode(&self, input: Input<'t>) -> Result<Vec<T>, DecodeError> {
        FromIteratorCodec::<&InnerCodec>::new(&self.0).decode(input)
    }

    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos)
        -> Result<(), DescriptorMismatch>
    {
        let inner = FromIteratorCodec::<&InnerCodec>::new(&self.0);
        <FromIteratorCodec::<&InnerCodec> as Codec<'t, Vec<T>>>::check_descriptor(&inner, ctx, type_pos)
    }
    fn encode(&self, output: &mut crate::serialization::Output, val: &Vec<T>) -> Result<(), crate::errors::EncodeError> {
        todo!()
    }
}