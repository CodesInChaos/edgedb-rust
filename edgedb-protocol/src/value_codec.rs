use std::any::type_name;
use std::convert::{TryInto, TryFrom};
use std::fmt;
use std::str;
use std::sync::Arc;
use std::collections::HashSet;

use uuid::Uuid as UuidVal;
use snafu::{ensure, OptionExt};

use crate::descriptors::{self, Descriptor, TypePos};
use crate::errors::{self, CodecError, DecodeError, EncodeError};
use crate::value::{Value, NamedTupleShape, ObjectShape, ObjectShapeInfo, NamedTupleShapeInfo, ShapeElement, TupleElement, EnumValue};
use crate::serialization::decode_composite::{DecodeTupleLike, DecodeArrayLike, DecodeInputTuple};
use crate::serialization::encode_composite::{EncodeTupleLike, EncodeArrayLike, EncodeInputTuple};
use crate::serialization::{Codec as _, ScalarCodec, Input, Output};
use crate::serialization::type_ids::*;


pub trait Codec: fmt::Debug + Send + Sync + 'static {
    fn decode(&self, input: Input) -> Result<Value, DecodeError>;
    fn encode(&self, output: &mut Output, value: &Value)
        -> Result<(), EncodeError>;
}

#[derive(Debug)]
struct Nothing;

#[derive(Debug)]
struct Object {
    shape: ObjectShape,
    codecs: Vec<Arc<dyn Codec>>,
}

#[derive(Debug)]
struct Set {
    element: Arc<dyn Codec>,
}

#[derive(Debug)]
struct Scalar {
    inner: Arc<dyn Codec>,
}

#[derive(Debug)]
struct Tuple {
    elements: Vec<Arc<dyn Codec>>,
}

#[derive(Debug)]
struct InputTuple {
    elements: Vec<Arc<dyn Codec>>,
}

#[derive(Debug)]
struct NamedTuple {
    shape: NamedTupleShape,
    codecs: Vec<Arc<dyn Codec>>,
}

#[derive(Debug)]
struct InputNamedTuple {
    shape: NamedTupleShape,
    codecs: Vec<Arc<dyn Codec>>,
}

#[derive(Debug)]
struct Array {
    element: Arc<dyn Codec>,
}

#[derive(Debug)]
struct Enum {
    members: HashSet<Arc<str>>,
}

struct CodecBuilder<'a> {
    input: bool,
    descriptors: &'a [Descriptor],
}

impl<'a> CodecBuilder<'a> {
    fn build(&self, pos: TypePos) -> Result<Arc<dyn Codec>, CodecError> {
        use Descriptor as D;
        if let Some(item) = self.descriptors.get(pos.0 as usize) {
            match item {
                D::BaseScalar(base) => scalar_codec(&base.id),
                D::Set(d) => Ok(Arc::new(Set::build(d, self)?)),
                D::ObjectShape(d) => Ok(Arc::new(Object::build(d, self)?)),
                D::Scalar(d) => Ok(Arc::new(Scalar {
                    inner: self.build(d.base_type_pos)?,
                })),
                D::Tuple(d) => {
                    if self.input {
                        Ok(Arc::new(InputTuple::build(d, self)?))
                    } else {
                        Ok(Arc::new(Tuple::build(d, self)?))
                    }
                }
                D::NamedTuple(d) => {
                    if self.input {
                        Ok(Arc::new(InputNamedTuple::build(d, self)?))
                    } else {
                        Ok(Arc::new(NamedTuple::build(d, self)?))
                    }
                }
                D::Array(d) => Ok(Arc::new(Array {
                    element: self.build(d.type_pos)?,
                })),
                D::Enumeration(d) => Ok(Arc::new(Enum {
                    members: d.members.iter().map(|x| x[..].into()).collect(),
                })),
                // type annotations are stripped from codecs array before
                // bilding a codec
                D::TypeAnnotation(..) => unreachable!(),
            }
        } else {
            return errors::UnexpectedTypePos { position: pos.0 }.fail()?;
        }
    }
}

pub fn build_codec(root_pos: Option<TypePos>,
    descriptors: &[Descriptor])
    -> Result<Arc<dyn Codec>, CodecError>
{
    let dec = CodecBuilder { input: false, descriptors };
    match root_pos {
        Some(pos) => dec.build(pos),
        None => Ok(Arc::new(Nothing {})),
    }
}

pub fn build_input_codec(root_pos: Option<TypePos>,
    descriptors: &[Descriptor])
    -> Result<Arc<dyn Codec>, CodecError>
{
    let dec = CodecBuilder { input: true, descriptors };
    match root_pos {
        Some(pos) => dec.build(pos),
        None => Ok(Arc::new(Nothing {})),
    }
}


pub fn scalar_codec(uuid: &UuidVal) -> Result<Arc<dyn Codec>, CodecError> {
    match *uuid {
        STD_UUID => Ok(Arc::new(Uuid {})),
        STD_STR => Ok(Arc::new(Str {})),
        STD_BYTES => Ok(Arc::new(Bytes {})),
        STD_INT16 => Ok(Arc::new(Int16 {})),
        STD_INT32 => Ok(Arc::new(Int32 {})),
        STD_INT64 => Ok(Arc::new(Int64 {})),
        STD_FLOAT32 => Ok(Arc::new(Float32 {})),
        STD_FLOAT64 => Ok(Arc::new(Float64 {})),
        STD_DECIMAL => Ok(Arc::new(Decimal {})),
        STD_BOOL => Ok(Arc::new(Bool {})),
        STD_DATETIME => Ok(Arc::new(Datetime {})),
        CAL_LOCAL_DATETIME => Ok(Arc::new(LocalDatetime {})),
        CAL_LOCAL_DATE => Ok(Arc::new(LocalDate {})),
        CAL_LOCAL_TIME => Ok(Arc::new(LocalTime {})),
        STD_DURATION => Ok(Arc::new(Duration {})),
        STD_JSON => Ok(Arc::new(Json {})),
        STD_BIGINT => Ok(Arc::new(BigInt {})),
        _ => return errors::UndefinedBaseScalar { uuid: uuid.clone() }.fail()?,
    }
}

macro_rules! implement_scalar_codec {
    ( $name:ident ) => (
        #[derive(Debug)]
        struct $name;

        impl Codec for $name {
            fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
                ScalarCodec::default().decode(buf).map(Value::$name)
            }
            fn encode(&self, output: &mut Output, val: &Value)
                -> Result<(), EncodeError>
            {
                match val {
                    Value::$name(val) => ScalarCodec::default().encode(output, val),
                    _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
                }
            }
        }
    )
}

implement_scalar_codec!{ Int16 }
implement_scalar_codec!{ Int32 }
implement_scalar_codec!{ Int64 }
implement_scalar_codec!{ Float32 }
implement_scalar_codec!{ Float64 }
implement_scalar_codec!{ Str }
implement_scalar_codec!{ Bytes }
implement_scalar_codec!{ Duration }
implement_scalar_codec!{ Uuid }
implement_scalar_codec!{ Decimal }
implement_scalar_codec!{ BigInt }
implement_scalar_codec!{ Bool }
implement_scalar_codec!{ Datetime }
implement_scalar_codec!{ LocalDatetime }
implement_scalar_codec!{ LocalDate }
implement_scalar_codec!{ LocalTime }
implement_scalar_codec!{ Json }

impl Codec for Nothing {
    fn decode(&self, _buf: Input) -> Result<Value, DecodeError> {
        Ok(Value::Nothing)
    }
    fn encode(&self, _output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        match val {
            Value::Nothing => Ok(()),
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        }
    }
}

impl Object {
    fn build(d: &descriptors::ObjectShapeDescriptor, dec: &CodecBuilder)
        -> Result<Object, CodecError>
    {
        Ok(Object {
            shape: d.elements.as_slice().into(),
            codecs: d.elements.iter()
                .map(|e| dec.build(e.type_pos))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Tuple {
    fn build(d: &descriptors::TupleTypeDescriptor, dec: &CodecBuilder)
        -> Result<Tuple, CodecError>
    {
        return Ok(Tuple {
            elements: d.element_types.iter()
                .map(|&t| dec.build(t))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl InputTuple {
    fn build(d: &descriptors::TupleTypeDescriptor, dec: &CodecBuilder)
        -> Result<InputTuple, CodecError>
    {
        return Ok(InputTuple {
            elements: d.element_types.iter()
                .map(|&t| dec.build(t))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl NamedTuple {
    fn build(d: &descriptors::NamedTupleTypeDescriptor, dec: &CodecBuilder)
        -> Result<NamedTuple, CodecError>
    {
        Ok(NamedTuple {
            shape: d.elements.as_slice().into(),
            codecs: d.elements.iter()
                .map(|e| dec.build(e.type_pos))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl InputNamedTuple {
    fn build(d: &descriptors::NamedTupleTypeDescriptor, dec: &CodecBuilder)
        -> Result<InputNamedTuple, CodecError>
    {
        Ok(InputNamedTuple {
            shape: d.elements.as_slice().into(),
            codecs: d.elements.iter()
                .map(|e| dec.build(e.type_pos))
                .collect::<Result<_, _>>()?,
        })
    }
}

fn decode_input_tuple<'t>(mut elements:DecodeInputTuple, codecs:&Vec<Arc<dyn Codec>>) -> Result<Vec<Value>, DecodeError>{
    codecs
        .iter()
        .map(|codec|codec.decode(elements.read()?))
        .collect::<Result<Vec<Value>, DecodeError>>()
}

fn decode_tuple<'t>(mut elements:DecodeTupleLike, codecs:&Vec<Arc<dyn Codec>>) -> Result<Vec<Value>, DecodeError>{
    codecs
        .iter()
        .map(|codec| codec.decode(elements.read()?))
        .collect::<Result<Vec<Value>, DecodeError>>()
}

fn decode_array_like<'t>(elements: DecodeArrayLike<'t>, codec:&dyn Codec) -> Result<Vec<Value>, DecodeError>{
    elements
        .map(|element| codec.decode(element?))
        .collect::<Result<Vec<Value>, DecodeError>>()
}

impl Codec for Object {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        let mut elements = DecodeTupleLike::new_object(buf, self.codecs.len())?;
        let fields = self.codecs
            .iter()
            .map(|codec| codec.decode(elements.read()?).map(|element| Some(element)))
            .collect::<Result<Vec<Option<Value>>, DecodeError>>()?;

        Ok(Value::Object {
            shape: self.shape.clone(),
            fields,
        })
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        let (shape, fields) = match val {
            Value::Object { shape, fields } => (shape, fields),
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        };
        ensure!(shape == &self.shape, errors::ObjectShapeMismatch);
        ensure!(self.codecs.len() == fields.len(),
                errors::ObjectShapeMismatch);
        debug_assert_eq!(self.codecs.len(), shape.0.elements.len());
        let elements = EncodeTupleLike::new(output);

        elements.finish();
        buf.reserve(4 + 8*self.codecs.len());
        buf.put_u32(self.codecs.len().try_into()
                    .ok().context(errors::TooManyElements)?);
        for (codec, field) in self.codecs.iter().zip(fields) {
            elements.write(|output|
            match field {
                Some(v) => {
                    codec.encode(buf, v)?;
                }
                None => {
                    buf.put_i32(-1);
                }
            }
        }
        Ok(())
    }
}

impl<'a> From<&'a [descriptors::ShapeElement]> for ObjectShape {
    fn from(shape: &'a [descriptors::ShapeElement]) -> ObjectShape {
        ObjectShape(Arc::new(ObjectShapeInfo {
                elements: shape.iter().map(|e| {
                    let descriptors::ShapeElement {
                        flag_implicit,
                        flag_link_property,
                        flag_link,
                        name,
                        type_pos: _,
                    } = e;
                    ShapeElement {
                        flag_implicit: *flag_implicit,
                        flag_link_property: *flag_link_property,
                        flag_link: *flag_link,
                        name: name.clone(),
                    }
                }).collect(),
            }))
    }
}

impl<'a> From<&'a [descriptors::TupleElement]> for NamedTupleShape {
    fn from(shape: &'a [descriptors::TupleElement]) -> NamedTupleShape {
        NamedTupleShape(Arc::new(NamedTupleShapeInfo {
                elements: shape.iter().map(|e| {
                    let descriptors::TupleElement {
                        name,
                        type_pos: _,
                    } = e;
                    TupleElement {
                        name: name.clone(),
                    }
                }).collect(),
            }))
    }
}

impl Set {
    fn build(d: &descriptors::SetDescriptor, dec: &CodecBuilder)
        -> Result<Set, CodecError>
    {
        Ok(Set {
            element: dec.build(d.type_pos)?,
        })
    }
}

impl Codec for Set {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        let elements = DecodeArrayLike::new_set(buf)?;
        let items = decode_array_like(elements, &*self.element)?;
        Ok(Value::Set(items))
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        let items = match val {
            Value::Set(items) => items,
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        };
        if items.is_empty() {
            buf.reserve(12);
            buf.put_u32(0);  // ndims
            buf.put_u32(0);  // reserved0
            buf.put_u32(0);  // reserved1
            return Ok(());
        }
        buf.reserve(20);
        buf.put_u32(1);  // ndims
        buf.put_u32(0);  // reserved0
        buf.put_u32(0);  // reserved1
        buf.put_u32(items.len().try_into().ok()
            .context(errors::ArrayTooLong)?);
        buf.put_u32(1);  // lower
        for item in items {
            buf.reserve(4);
            let pos = buf.len();
            buf.put_u32(0);  // replaced after serializing a value
            self.element.encode(buf, item)?;
            let len = buf.len()-pos-4;
            buf[pos..pos+4].copy_from_slice(&u32::try_from(len)
                    .ok().context(errors::ElementTooLong)?
                    .to_be_bytes());
        }
        Ok(())
    }
}

impl Codec for Scalar {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        self.inner.decode(buf)
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        self.inner.encode(output, val)
    }
}

impl Codec for Tuple {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        let elements = DecodeTupleLike::new_object(buf, self.elements.len())?;
        let items = decode_tuple(elements, &self.elements)?;
        return Ok(Value::Tuple(items))
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        let items = match val {
            Value::Tuple(items) => items,
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        };
        ensure!(self.elements.len() == items.len(),
            errors::TupleShapeMismatch);
        buf.reserve(4 + 8*self.elements.len());
        buf.put_u32(self.elements.len().try_into()
                    .ok().context(errors::TooManyElements)?);
        for (codec, item) in self.elements.iter().zip(items) {
            buf.reserve(8);
            buf.put_u32(0);
            let pos = buf.len();
            buf.put_u32(0);  // replaced after serializing a value
            codec.encode(buf, item)?;
            let len = buf.len()-pos-4;
            buf[pos..pos+4].copy_from_slice(&u32::try_from(len)
                    .ok().context(errors::ElementTooLong)?
                    .to_be_bytes());
        }
        Ok(())
    }
}

impl Codec for InputTuple {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        let elements = DecodeInputTuple::with_count(buf, self.elements.len())?;
        let items = decode_input_tuple(elements, &self.elements)?;
        Ok(Value::Tuple(items))
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        let items = match val {
            Value::Tuple(items) => items,
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        };
        ensure!(self.elements.len() == items.len(),
            errors::TupleShapeMismatch);
        buf.reserve(4 + 4*self.elements.len());
        buf.put_u32(self.elements.len().try_into()
                    .ok().context(errors::TooManyElements)?);
        for (codec, item) in self.elements.iter().zip(items) {
            buf.reserve(4);
            let pos = buf.len();
            buf.put_u32(0);  // replaced after serializing a value
            codec.encode(buf, item)?;
            let len = buf.len()-pos-4;
            buf[pos..pos+4].copy_from_slice(&u32::try_from(len)
                    .ok().context(errors::ElementTooLong)?
                    .to_be_bytes());
        }
        Ok(())
    }
}

impl Codec for NamedTuple {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        let elements = DecodeTupleLike::new_tuple(buf, self.codecs.len())?;
        let fields = decode_tuple(elements, &self.codecs)?;
        return Ok(Value::NamedTuple {
            shape: self.shape.clone(),
            fields,
        })
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        let (shape, fields) = match val {
            Value::NamedTuple { shape, fields } => (shape, fields),
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        };
        ensure!(shape == &self.shape, errors::TupleShapeMismatch);
        ensure!(self.codecs.len() == fields.len(),
                errors::ObjectShapeMismatch);

        debug_assert_eq!(self.codecs.len(), shape.0.elements.len());
        buf.reserve(4 + 8*self.codecs.len());
        buf.put_u32(self.codecs.len().try_into()
                    .ok().context(errors::TooManyElements)?);
        for (codec, field) in self.codecs.iter().zip(fields) {
            buf.reserve(8);
            buf.put_u32(0);
            let pos = buf.len();
            buf.put_u32(0);  // replaced after serializing a value
            codec.encode(buf, field)?;
            let len = buf.len()-pos-4;
            buf[pos..pos+4].copy_from_slice(&u32::try_from(len)
                    .ok().context(errors::ElementTooLong)?
                    .to_be_bytes());
        }
        Ok(())
    }
}

impl Codec for InputNamedTuple {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        let elements = DecodeInputTuple::with_count(buf, self.codecs.len())?;
        let fields = decode_input_tuple(elements, &self.codecs)?;
        Ok(Value::NamedTuple {
            shape: self.shape.clone(),
            fields,
        })
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        let (shape, fields) = match val {
            Value::NamedTuple { shape, fields } => (shape, fields),
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        };
        ensure!(shape == &self.shape, errors::TupleShapeMismatch);
        ensure!(self.codecs.len() == fields.len(),
                errors::ObjectShapeMismatch);
        debug_assert_eq!(self.codecs.len(), shape.0.elements.len());
        buf.reserve(4 + 8*self.codecs.len());
        buf.put_u32(self.codecs.len().try_into()
                    .ok().context(errors::TooManyElements)?);
        for (codec, field) in self.codecs.iter().zip(fields) {
            buf.reserve(4);
            let pos = buf.len();
            buf.put_u32(0);  // replaced after serializing a value
            codec.encode(buf, field)?;
            let len = buf.len()-pos-4;
            buf[pos..pos+4].copy_from_slice(&u32::try_from(len)
                    .ok().context(errors::ElementTooLong)?
                    .to_be_bytes());
        }
        Ok(())
    }
}

impl Codec for Array {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        let elements = DecodeArrayLike::new_array(buf)?;
        let items = decode_array_like(elements, &*self.element)?;
        Ok(Value::Array(items))
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        let items = match val {
            Value::Array(items) => items,
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        };
        if items.is_empty() {
            buf.reserve(12);
            buf.put_u32(0);  // ndims
            buf.put_u32(0);  // reserved0
            buf.put_u32(0);  // reserved1
            return Ok(());
        }
        buf.reserve(20);
        buf.put_u32(1);  // ndims
        buf.put_u32(0);  // reserved0
        buf.put_u32(0);  // reserved1
        buf.put_u32(items.len().try_into().ok()
            .context(errors::ArrayTooLong)?);
        buf.put_u32(1);  // lower
        for item in items {
            buf.reserve(4);
            let pos = buf.len();
            buf.put_u32(0);  // replaced after serializing a value
            self.element.encode(buf, item)?;
            let len = buf.len()-pos-4;
            buf[pos..pos+4].copy_from_slice(&u32::try_from(len)
                    .ok().context(errors::ElementTooLong)?
                    .to_be_bytes());
        }
        Ok(())
    }
}

impl Codec for Enum {
    fn decode(&self, buf: Input) -> Result<Value, DecodeError> {
        let val : &str = ScalarCodec::default().decode(buf)?;
        let val = self.members.get(val)
            .context(errors::ExtraEnumValue)?;
        Ok(Value::Enum(EnumValue(val.clone())))
    }
    fn encode(&self, output: &mut Output, val: &Value)
        -> Result<(), EncodeError>
    {
        match val {
            Value::Enum(val) => {
                ensure!(self.members.get(&val.0).is_some(), errors::MissingEnumValue);
                ScalarCodec::default().encode(output, &val.0.as_ref())
            },
            _ => Err(errors::invalid_value(type_name::<Self>(), val))?,
        }
    }
}
