pub mod decode_composite;
pub mod codecs;
pub(crate) mod type_ids;
use snafu::ensure;
use bytes::BytesMut;

use crate::errors::{self, DecodeError};

pub use self::codecs::{Codec, DefaultCodec, ScalarCodec, CompositeCodec};

pub struct Input<'t>(pub Option<&'t [u8]>);
pub struct Output(BytesMut);

impl<'t> Input<'t> {
    pub(self) fn bytes(self) -> Result<&'t [u8], DecodeError> {
        ensure!(self.0.is_some(), errors::MissingRequiredElement);
        Ok(self.0.unwrap())
    }

    pub fn new(bytes:&'t [u8]) -> Input<'t> {
        Input(Some(bytes))
    }
}

impl Output {
    pub(crate) fn buf(&mut self) -> &mut BytesMut {
        &mut self.0
    }
}