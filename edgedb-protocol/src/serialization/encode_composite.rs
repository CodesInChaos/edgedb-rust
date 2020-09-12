use crate::errors::EncodeError;
use crate::serialization::Output;
use inner::EncodeCompositeInner;

pub(crate) struct EncodeTupleLike<'t> {
    inner: EncodeCompositeInner<'t>,
}

impl<'t> EncodeTupleLike<'t> {
    pub fn new(output: &'t mut Output) -> EncodeTupleLike<'t> {
        let inner = EncodeCompositeInner::new_tuple_like(output);
        EncodeTupleLike { inner }
    }

    pub fn write(
        &'t mut self,
        f: impl FnOnce(&'t mut Output) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.inner.write_tuple_like_element(f)
	}
	
	pub fn write_null(&'t mut self) {
        self.inner.write_null()
    }

    pub fn finish(self) {
        self.inner.finish_tuple_like()
    }
}

pub(crate) struct EncodeInputTuple<'t> {
    inner: EncodeCompositeInner<'t>,
}

impl<'t> EncodeInputTuple<'t> {
    pub fn new(output: &'t mut Output) -> EncodeInputTuple<'t> {
        let inner = EncodeCompositeInner::new_tuple_like(output);
        EncodeInputTuple { inner }
    }

    pub fn write(
        &'t mut self,
        f: impl FnOnce(&'t mut Output) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.inner.write_array_like_element(f)
    }

    pub fn finish(self) {
        self.inner.finish_tuple_like()
    }
}

pub(crate) struct EncodeArrayLike<'t> {
    inner: EncodeCompositeInner<'t>,
}

impl<'t> EncodeArrayLike<'t> {
    pub fn new(output: &'t mut Output) ->EncodeArrayLike<'t> {
        let inner = EncodeCompositeInner::new_array_like(output);
        EncodeArrayLike { inner }
    }

    pub fn write(
        &'t mut self,
        f: impl FnOnce(&'t mut Output) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.inner.filled_array();
        self.inner.write_array_like_element(f)
    }

    pub fn finish(self) {
        self.inner.finish_array_like()
    }
}

mod inner {
    use crate::errors::{self, EncodeError};
    use crate::serialization::Output;
    use bytes::{BufMut, BytesMut};
    use snafu::OptionExt;
    use std::convert::TryFrom;

    pub(super) struct EncodeCompositeInner<'t> {
        output: &'t mut Output,
        position: usize,
        count: u32,
    }

    impl<'t> EncodeCompositeInner<'t> {
        fn output(&'t mut self) -> &'t mut Output {
            &mut self.output
        }

        fn buf(&'t mut self) -> &'t mut BytesMut {
            self.output().buf()
        }

        pub fn new_tuple_like(output: &'t mut Output) -> EncodeCompositeInner<'t> {
            let result = EncodeCompositeInner {
                output,
                position: output.buf().len(),
                count: 0,
            };

            let buf = result.buf();
            buf.put_u32(0); // count - filled in finish_tuple_like

            result
        }

        pub fn new_array_like(output: &'t mut Output) -> EncodeCompositeInner<'t> {
            EncodeCompositeInner {
                output,
                position: output.buf().len(),
                count: 0,
            }
        }

        pub fn filled_array(&'t mut self) {
            let buf = self.buf();
            if self.count == 0 {
                buf.reserve(20);
                buf.put_u32(1); // ndims
                buf.put_u32(0); // reserved0
                buf.put_u32(0); // reserved1
                buf.put_u32(0); // upper - filled in finish_array_like
                buf.put_u32(1); // lower
            }
        }

        pub fn finish_tuple_like(self) {
            let buf = self.buf();
            buf[self.position..self.position + 4].copy_from_slice(&self.count.to_be_bytes());
        }

        pub fn finish_array_like(self) {
            let buf = self.buf();
            if self.count == 0 {
                buf.reserve(12);
                buf.put_u32(0); // ndims
                buf.put_u32(0); // reserved0
                buf.put_u32(0); // reserved1
            } else {
                buf[self.position + 12..self.position + 16]
                    .copy_from_slice(&self.count.to_be_bytes());
            }
		}
		
		pub fn write_null(&'t mut self) {
            let buf = self.buf();
            buf.reserve(8);
			buf.put_u32(0); // reserved
			buf.put_i32(-1); // count
		}

        pub fn write_tuple_like_element(
            &'t mut self,
            f: impl FnOnce(&'t mut Output) -> Result<(), EncodeError>,
        ) -> Result<(), EncodeError> {
            let buf = self.buf();
            buf.reserve(8);
            buf.put_u32(0); // reserved
            self.write_array_like_element(f)
        }

        pub fn write_array_like_element(
            &'t mut self,
            f: impl FnOnce(&'t mut Output) -> Result<(), EncodeError>,
        ) -> Result<(), EncodeError> {
            let buf = self.buf();
            buf.reserve(4);
            buf.put_u32(0); // replaced after serializing a value
            let pos = buf.len();
            f(self.output())?;
            let len = buf.len() - pos;
            buf[pos..pos + 4].copy_from_slice(
                &u32::try_from(len)
                    .ok()
                    .context(errors::ElementTooLong)?
                    .to_be_bytes(),
            );
            Ok(())
        }
    }
}
