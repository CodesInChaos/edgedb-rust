
use snafu::{Snafu};

use crate::errors::DecodeError;
use crate::descriptors::{Descriptor, TypePos};
use crate::serialization::{Input, Codec, ScalarCodec};

pub trait Queryable: Sized {
    fn decode(buf: Input) -> Result<Self, DecodeError>;
    fn check_descriptor(ctx: &DescriptorContext, type_pos: TypePos)
        -> Result<(), DescriptorMismatch>;
}

#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum DescriptorMismatch {
    #[snafu(display("unexpected type {}, expected {}", unexpected, expected))]
    WrongType { unexpected: String, expected: String },
    #[snafu(display("unexpected field {}, expected {}", unexpected, expected))]
    WrongField { unexpected: String, expected: String },
    #[snafu(display("expected {} fields, got {}", expected, unexpected))]
    FieldNumber { unexpected: usize, expected: usize },
    #[snafu(display("expected {}", expected))]
    Expected { expected: String },
    #[snafu(display("invalid type descriptor"))]
    InvalidDescriptor,
}

pub struct DescriptorContext<'a> {
    descriptors: &'a [Descriptor],
}

impl DescriptorContext<'_> {
    pub(crate) fn new(descriptors: &[Descriptor]) -> DescriptorContext {
        DescriptorContext { descriptors }
    }
    pub fn get(&self, type_pos: TypePos)
        -> Result<&Descriptor, DescriptorMismatch>
    {
        self.descriptors.get(type_pos.0 as usize)
            .ok_or(DescriptorMismatch::InvalidDescriptor)
    }
    pub fn wrong_type(&self, descriptor: &Descriptor, expected: &str)
        -> DescriptorMismatch
    {
        DescriptorMismatch::WrongType {
            // TODO(tailhook) human-readable type description
            unexpected: format!("{:?}", descriptor),
            expected: expected.into(),
        }
    }
    pub fn field_number(&self, expected: usize, unexpected: usize)
        -> DescriptorMismatch
    {
        DescriptorMismatch::FieldNumber { expected, unexpected }
    }
    pub fn wrong_field(&self, expected: &str, unexpected: &str)
        -> DescriptorMismatch
    {
        DescriptorMismatch::WrongField {
            expected: expected.into(),
            unexpected: unexpected.into(),
        }
    }
    pub fn expected(&self, expected: &str)
        -> DescriptorMismatch
    {
        DescriptorMismatch::Expected { expected: expected.into() }
    }
}

impl<T> Queryable for T
    where for<'t> ScalarCodec: Codec<'t, T>
{
    fn decode(buf: Input) -> Result<Self, DecodeError> {
        ScalarCodec::default().decode(buf)
    }
    fn check_descriptor(ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
        ScalarCodec::default().check_descriptor(ctx, type_pos)
    }
}

// impl<T> Queryable for T
//     where for<'t> CompositeCodec<DefaultCodec>: Codec<'t, T>
// {
//     fn decode(buf: Input) -> Result<Self, DecodeError> {
//         CompositeCodec::<DefaultCodec>::decode(buf)
//     }
//     fn check_descriptor(ctx: &DescriptorContext, type_pos: TypePos) -> Result<(), DescriptorMismatch> {
//         CompositeCodec::<DefaultCodec>::check_descriptor(ctx, type_pos)
//     }
// }