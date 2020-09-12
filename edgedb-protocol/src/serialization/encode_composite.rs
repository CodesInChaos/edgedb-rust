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
        &mut self,
        f: impl FnOnce(&mut Output) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.inner.write_tuple_like_element(f)
	}
	
	pub fn write_null(&mut self) {
        self.inner.write_null()
    }
}

impl<'t> Drop for EncodeTupleLike<'t> {
    fn drop(&mut self) {
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
        &mut self,
        f: impl FnOnce(&mut Output) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.inner.write_array_like_element(f)
    }
}

impl<'t> Drop for EncodeInputTuple<'t> {
    fn drop(&mut self) {
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
        &mut self,
        f: impl FnOnce(&mut Output) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.inner.filled_array();
        self.inner.write_array_like_element(f)
    }
}

impl<'t> Drop for EncodeArrayLike<'t> {
    fn drop(&mut self) {
        self.inner.finish_array_like()
    }
}

mod inner {
    use crate::errors::{self, EncodeError};
    use crate::serialization::Output;
    use bytes::BufMut;
    use snafu::OptionExt;
    use std::convert::TryFrom;

    pub(super) struct EncodeCompositeInner<'t> {
        output: &'t mut Output,
        position: usize,
        count: u32,
    }

    impl<'t> EncodeCompositeInner<'t> {
        fn output(&mut self) -> &mut Output {
            &mut self.output
        }

        pub fn new_tuple_like(output: &'t mut Output) -> EncodeCompositeInner<'t> {
            let position = output.len();
            let mut result = EncodeCompositeInner {
                output,
                position,
                count: 0,
            };

            let buf = result.output();
            buf.put_u32(0); // count - filled in finish_tuple_like

            result
        }

        pub fn new_array_like(output: &'t mut Output) -> EncodeCompositeInner<'t> {
            let position = output.len(); 
            EncodeCompositeInner {
                output,
                position: position,
                count: 0,
            }
        }

        pub fn filled_array(&mut self) {
            let count = self.count;
            let buf = self.output();
            if count == 0 {
                buf.reserve(20);
                buf.put_u32(1); // ndims
                buf.put_u32(0); // reserved0
                buf.put_u32(0); // reserved1
                buf.put_u32(0); // upper - filled in finish_array_like
                buf.put_u32(1); // lower
            }
        }

        pub fn finish_tuple_like(&mut self) {
            let position = self.position;
            let count = self.count;
            let buf = self.output();
            buf[position..position + 4].copy_from_slice(&count.to_be_bytes());
        }

        pub fn finish_array_like(&mut self) {
            let position = self.position;
            let count = self.count;
            let buf = self.output();
            if count == 0 {
                buf.reserve(12);
                buf.put_u32(0); // ndims
                buf.put_u32(0); // reserved0
                buf.put_u32(0); // reserved1
            } else {
                buf[position + 12..position + 16]
                    .copy_from_slice(&count.to_be_bytes());
            }
		}
		
		pub fn write_null(&mut self) {
            let buf = self.output();
            buf.reserve(8);
			buf.put_u32(0); // reserved
			buf.put_i32(-1); // count
		}

        pub fn write_tuple_like_element(
            &mut self,
            f: impl FnOnce(&mut Output) -> Result<(), EncodeError>,
        ) -> Result<(), EncodeError> {
            let buf = self.output();
            buf.reserve(8);
            buf.put_u32(0); // reserved
            self.write_array_like_element(f)
        }

        pub fn write_array_like_element(
            &mut self,
            f: impl FnOnce(&mut Output) -> Result<(), EncodeError>,
        ) -> Result<(), EncodeError> {
            let buf = self.output();
            buf.reserve(4);
            buf.put_u32(0); // replaced after serializing a value
            let pos = buf.len();
            f(self.output())?;
            let buf = self.output();
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
