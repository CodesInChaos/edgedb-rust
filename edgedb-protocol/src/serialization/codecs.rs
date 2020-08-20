mod composite;
mod scalars;
mod tuples;
mod from_iterator;

use crate::serialization::{Input, Output};
use crate::errors::{DecodeError, EncodeError};
use crate::descriptors::TypePos;
use crate::queryable::{Queryable, DescriptorContext, DescriptorMismatch};


pub use composite::CompositeCodec;
pub use scalars::ScalarCodec;
use from_iterator::FromIteratorCodec;

#[derive(Debug)]
pub struct DefaultCodec;

pub trait Codec<'t, T>: std::fmt::Debug + Send + Sync {
    fn decode(&self, input:Input<'t>) -> Result<T, DecodeError>;
    fn encode(&self, output: &mut Output, val: &T) -> Result<(), EncodeError>;
    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch>;
}

impl<'t, T, C: Codec<'t, T>> Codec<'t, T> for &C {
    fn decode(&self, input:Input<'t>) -> Result<T, DecodeError> {
        (*self).decode(input)
    }

    fn check_descriptor(&self, ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        (*self).check_descriptor(ctx, type_pos)
    }

    fn encode(&self, output: &mut Output, val: &T) -> Result<(), EncodeError> {
        (*self).encode(output, val)
    }
}

impl<'t,T:Queryable> Codec<'t, T> for DefaultCodec {
    fn decode(&self, buf:Input<'t>) -> Result<T, crate::errors::DecodeError> {
        T::decode(buf)
    }
    fn encode(&self, _output: &mut Output, _val: &T) -> Result<(), crate::errors::EncodeError> {
        todo!()
    }
    fn check_descriptor(&self, ctx: &crate::queryable::DescriptorContext, type_pos: crate::descriptors::TypePos) -> Result<(), crate::queryable::DescriptorMismatch> {
        T::check_descriptor(ctx, type_pos)
    }    
}