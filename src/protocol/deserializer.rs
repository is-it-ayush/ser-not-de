use serde::{
    de::{EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess},
    Deserialize, Deserializer,
};

use super::{
    error::Error,
    serializer::{
        BYTE_DELIMITER, ENUM_DELIMITER, MAP_DELIMITER, MAP_KEY_DELIMITER, MAP_VALUE_DELIMITER,
        MAP_VALUE_SEPARATOR, SEQ_DELIMITER, SEQ_VALUE_DELIMITER, STRING_DELIMITER, UNIT,
    },
};

/// - The seperators are u8.
/// - The seperators need to be unique among serde-data-model types.
/// - Primitive types are serialized as is.
///     - bool: 0 -> false, 1 -> true (1 byte)
///     - i8, i16, i32, i64: as is.
///     - u8, u16, u32, u64: as is.
///     - f32, f64: as is.
///     - char: as u32 (4 bytes)
///
/// - String, Bytes, Unit, Option are serialized as:
///     - str: STRING_DELIMITER + bytes + STRING_DELIMITER
///     - bytes: BYTE_DELIMITER + bytes + BYTE_DELIMITER
///     - unit: UNIT (null)
///     - option: None -> unit(), Some -> self
///
/// - Structs are serialized as:
///     - unit_struct: unit()
///     - newtype_struct: self
///     - tuple_struct: seq()
///
/// - Enums are serialized as:
///     - unit_variant: ENUM_DELIMITER + variant_index
///     - newtype_variant: ENUM_DELIMITER + variant_index + self
///     - tuple_variant: ENUM_DELIMITER + variant_index + tuple()
///     - struct_variant: ENUM_DELIMITER + variant_index + struct()
///
/// - Sequences are serialized as:
///     - SEQ_DELIMITER + value_1 + SEQ_VALUE_DELIMITER + value_2 + SEQ_VALUE_DELIMITER + ... + SEQ_DELIMITER
///
/// - Maps are serialized as:
///     - MAP_DELIMITER + key_1 + MAP_KEY_DELIMITER + value_1 + MAP_VALUE_DELIMITER + key_2 + MAP_KEY_DELIMITER + value_2 + MAP_VALUE_DELIMITER + ... + MAP_DELIMITER
///
/// - Tuples and Structs are serialized as:
///     - tuple: seq()
///     - struct: map()

#[derive(Debug)]
struct CustomDeserializer<'de> {
    data: &'de [u8],
}

pub fn from_bytes<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let mut deserializer = CustomDeserializer { data: bytes };
    let deserialized = T::deserialize(&mut deserializer)?;
    Ok(deserialized)
}

impl<'de> CustomDeserializer<'de> {
    /// Get the last byte from the data.
    pub fn peek_byte(&self) -> Result<&u8, Error> {
        let data = self.data.first().ok_or(Error::NoByte)?;
        Ok(data)
    }
    /// Grab the next byte from the data and remove it.
    pub fn eat_byte(&mut self) -> Result<u8, Error> {
        let byte = *self.peek_byte()?;
        self.data = &self.data[1..];
        Ok(byte)
    }
    /// Grab the next 'n' bytes from the data and remove them.
    pub fn eat_bytes(&mut self, n: usize) -> Result<&[u8], Error> {
        let bytes = &self.data[..n];
        self.data = &self.data[n..];
        Ok(bytes)
    }

    /// Parser Methods

    /// Parses a boolean value from the input.
    pub fn parse_bool(&mut self) -> Result<bool, Error> {
        Ok(self.eat_byte()? != 0)
    }
    /// Parses an unsigned integer value from the input.
    pub fn parse_unsigned<T>(&mut self) -> Result<T, Error>
    where
        T: TryFrom<u8> + TryFrom<u16> + TryFrom<u32> + TryFrom<u64>,
    {
        let length = std::mem::size_of::<T>();
        if self.data.len() < length {
            return Err(Error::UnexpectedEOF);
        }
        match length {
            1 => {
                let byte = self.eat_byte()?;
                u8::from_le_bytes([byte])
                    .try_into()
                    .map_err(|_| Error::ConversionError)
            }
            2 => {
                let bytes = self.eat_bytes(length)?;
                u16::from_le_bytes([bytes[0], bytes[1]])
                    .try_into()
                    .map_err(|_| Error::ConversionError)
            }
            4 => {
                let bytes = self.eat_bytes(length)?;
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                    .try_into()
                    .map_err(|_| Error::ConversionError)
            }
            8 => {
                let bytes = self.eat_bytes(length)?;
                u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ])
                .try_into()
                .map_err(|_| Error::ConversionError)
            }
            _ => Err(Error::InvalidTypeSize),
        }
    }
    /// Parses a signed integer value from the input.
    pub fn parse_signed<T>(&mut self) -> Result<T, Error>
    where
        T: TryFrom<i8> + TryFrom<i16> + TryFrom<i32> + TryFrom<i64>,
    {
        let length = std::mem::size_of::<T>();
        if self.data.len() < length {
            return Err(Error::UnexpectedEOF);
        }
        match length {
            1 => {
                let byte = self.eat_byte()?;
                i8::from_le_bytes([byte])
                    .try_into()
                    .map_err(|_| Error::ConversionError)
            }
            2 => {
                let bytes = self.eat_bytes(length)?;
                i16::from_le_bytes([bytes[0], bytes[1]])
                    .try_into()
                    .map_err(|_| Error::ConversionError)
            }
            4 => {
                let bytes = self.eat_bytes(length)?;
                i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                    .try_into()
                    .map_err(|_| Error::ConversionError)
            }
            8 => {
                let bytes = self.eat_bytes(length)?;
                i64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ])
                .try_into()
                .map_err(|_| Error::ConversionError)
            }
            _ => Err(Error::InvalidTypeSize),
        }
    }
    /// Parses a 32-bit floating point value from the input.
    pub fn parse_f32(&mut self) -> Result<f32, Error> {
        let bytes = self.eat_bytes(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    /// Parses a 64-bit floating point value from the input.
    pub fn parse_f64(&mut self) -> Result<f64, Error> {
        let bytes = self.eat_bytes(8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
    /// Parses a character value from the input.
    pub fn parse_char(&mut self) -> Result<char, Error> {
        let value = self.parse_unsigned::<u32>()?;
        Ok(std::char::from_u32(value).unwrap())
    }

    /// Parses a string value from the input.
    pub fn parse_str(&mut self, bytes: &mut Vec<u8>) -> Result<String, Error> {
        loop {
            let byte = self.eat_byte()?;
            if byte == STRING_DELIMITER {
                break;
            }
            bytes.push(byte);
        }
        Ok(String::from_utf8(bytes.clone()).map_err(|_| Error::ConversionError)?)
    }

    /// Parses a byte buffer from the input.
    pub fn parse_bytes(&mut self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        loop {
            let byte = self.eat_byte()?;
            if byte == STRING_DELIMITER {
                break;
            }
            bytes.push(byte);
        }
        Ok(())
    }
}

impl<'de, 'a> Deserializer<'de> for &'a mut CustomDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::UnsupportedCall("deserialize_any".to_string()))
    }

    /// Primitve Types Deserialization. They are serialized as is (LE byte order).
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i8(self.parse_signed::<i8>()?)
    }
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i16(self.parse_signed::<i16>()?)
    }
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i32(self.parse_signed::<i32>()?)
    }
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed::<i64>()?)
    }
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(self.parse_unsigned::<u8>()?)
    }
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u16(self.parse_unsigned::<u16>()?)
    }
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned::<u32>()?)
    }
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned::<u64>()?)
    }
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse_f32()?)
    }
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse_f64()?)
    }
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_char(self.parse_char()?)
    }

    /// String Deserialization. They are serialized as STRING_DELIMITER + bytes + STRING_DELIMITER.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parse_unsigned::<u8>()? {
            STRING_DELIMITER => {
                let mut bytes = Vec::new();
                visitor.visit_str(self.parse_str(&mut bytes)?.as_str())
            }
            _ => Err(Error::ExpectedStringDelimiter),
        }
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parse_unsigned::<u8>()? {
            STRING_DELIMITER => {
                let mut bytes = Vec::new();
                visitor.visit_string(self.parse_str(&mut bytes)?.to_string())
            }
            _ => Err(Error::ExpectedStringDelimiter),
        }
    }

    /// Byte Deserialization. They are serialized as BYTE_DELIMITER + bytes + BYTE_DELIMITER.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parse_unsigned::<u8>()? {
            BYTE_DELIMITER => {
                let mut bytes = Vec::new();
                self.parse_bytes(&mut bytes)?;
                visitor.visit_bytes(&bytes)
            }
            _ => Err(Error::ExpectedByteDelimiter),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parse_unsigned::<u8>()? {
            BYTE_DELIMITER => {
                let mut bytes = Vec::new();
                self.parse_bytes(&mut bytes)?;
                visitor.visit_byte_buf(bytes)
            }
            _ => Err(Error::ExpectedByteDelimiter),
        }
    }

    /// Option Deserialization. They are serialized as None -> unit(), Some -> self.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.peek_byte()? {
            &UNIT => {
                self.eat_byte()?;
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }
    /// Unit Deserialization. They are serialized as UNIT.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parse_unsigned::<u8>()? {
            UNIT => visitor.visit_unit(),
            _ => Err(Error::ExpectedUnit),
        }
    }

    /// Struct Deserialization.
    /// - unit_struct: unit()
    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
    /// - newtype_struct: self
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }
    /// - tuple_struct: seq()
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    /// Enum Deserialization.
    /// - unit_variant: ENUM_DELIMITER + variant_index
    /// - newtype_variant: ENUM_DELIMITER + variant_index + self
    /// - tuple_variant: ENUM_DELIMITER + variant_index + tuple()
    /// - struct_variant: ENUM_DELIMITER + variant_index + struct()
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parse_unsigned::<u8>()? {
            ENUM_DELIMITER => visitor.visit_enum(self),
            _ => Err(Error::ExpectedEnumDelimiter),
        }
    }

    /// Seq & Map Deserialization.
    /// - seq: SEQ_DELIMITER + value_1 + SEQ_VALUE_DELIMITER + value_2 + SEQ_VALUE_DELIMITER + ... + SEQ_DELIMITER
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parse_unsigned::<u8>()? {
            SEQ_DELIMITER => {
                let value = visitor.visit_seq(MinimalSequenceDeserializer::new(self))?;
                if self.parse_unsigned::<u8>()? != SEQ_DELIMITER {
                    return Err(Error::ExpectedSeqDelimiter);
                }
                Ok(value)
            }
            _ => Err(Error::ExpectedSeqDelimiter),
        }
    }
    /// - map: MAP_DELIMITER + key_1 + MAP_KEY_DELIMITER + value_1 + MAP_VALUE_DELIMITER + key_2 + MAP_KEY_DELIMITER + value_2 + MAP_VALUE_DELIMITER + ... + MAP_DELIMITER
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parse_unsigned::<u8>()? {
            MAP_DELIMITER => {
                let value = visitor.visit_map(MinimalMapDeserializer::new(self))?;
                if self.parse_unsigned::<u8>()? != MAP_DELIMITER {
                    return Err(Error::ExpectedMapDelimiter);
                }
                Ok(value)
            }
            e => Err(Error::ExpectedMapDelimiter),
        }
    }

    /// Tuple & Struct Deserialization.
    /// - tuple: seq()
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
    /// - struct: map()
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::UnsupportedCall(
            "deserialize_ignored_any".to_string(),
        ))
    }
}

/// Enum Deserialization
/// ENUM_DELIMITER + variant_index + (depends on variant type; handled by VARIANT_ACCESS)
impl<'de, 'a> EnumAccess<'de> for &'a mut CustomDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    /// Get the next variant key from the data and remove it.
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        // ENUM_DELIMITER + variant_index + (depends on variant type; handled by variant_access)
        let key = self.parse_unsigned::<u32>()?;
        Ok((seed.deserialize(key.into_deserializer())?, self))
    }
}
impl<'de, 'a> VariantAccess<'de> for &'a mut CustomDeserializer<'de> {
    type Error = Error;

    /// - unit_variant: ENUM_DELIMITER + variant_index
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// - newtype_variant: ENUM_DELIMITER + variant_index + self
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    /// - tuple_variant: ENUM_DELIMITER + variant_index + tuple() => seq()
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    /// - struct_variant: ENUM_DELIMITER + variant_index + struct()
    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_struct("", fields, visitor)
    }
}

/// Sequence Deserialization: seq()
///     - SEQ_DELIMITER + value_1 + SEQ_VALUE_DELIMITER + value_2 + SEQ_VALUE_DELIMITER + ... + SEQ_DELIMITER
struct MinimalSequenceDeserializer<'a, 'de: 'a> {
    deserializer: &'a mut CustomDeserializer<'de>,
    first: bool,
}
impl<'a, 'de> MinimalSequenceDeserializer<'a, 'de> {
    pub fn new(deserializer: &'a mut CustomDeserializer<'de>) -> Self {
        Self {
            deserializer: deserializer,
            first: true,
        }
    }
}
impl<'de, 'a> SeqAccess<'de> for MinimalSequenceDeserializer<'a, 'de> {
    type Error = Error;

    // value_1 + SEQ_VALUE_DELIMITER + value_2 + SEQ_VALUE_DELIMITER + ... + SEQ_DELIMITER
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        // if at end of sequence; exit
        if self.deserializer.peek_byte()? == &SEQ_DELIMITER {
            return Ok(None);
        }
        // if not first and not at the end of sequence; eat SEQ_VALUE_DELIMITER
        if !self.first && self.deserializer.eat_byte()? != SEQ_VALUE_DELIMITER {
            return Err(Error::ExpectedSeqValueDelimiter);
        }
        // make not first; deserialize next element
        self.first = false;
        seed.deserialize(&mut *self.deserializer).map(Some)
    }
}

/// Map Deserialization: map()
///     - MAP_DELIMITER + key_1 + MAP_KEY_DELIMITER + value_1 + MAP_VALUE_DELIMITER + key_2 + MAP_KEY_DELIMITER + value_2 + MAP_VALUE_DELIMITER + ... + MAP_DELIMITER
struct MinimalMapDeserializer<'a, 'de: 'a> {
    deserializer: &'a mut CustomDeserializer<'de>,
    first: bool,
}
impl<'a, 'de> MinimalMapDeserializer<'a, 'de> {
    pub fn new(deserializer: &'a mut CustomDeserializer<'de>) -> Self {
        Self {
            deserializer: deserializer,
            first: true,
        }
    }
}
impl<'de, 'a> MapAccess<'de> for MinimalMapDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        // if at end of map; exit
        if self.deserializer.peek_byte()? == &MAP_DELIMITER {
            return Ok(None);
        }
        // if not first and not at the end of map; eat MAP_KEY_DELIMITER
        if !self.first && self.deserializer.eat_byte()? != MAP_VALUE_SEPARATOR {
            return Err(Error::ExpectedMapValueSeparator);
        }
        // make not first; deserialize next key_1
        self.first = false;
        if self.deserializer.parse_unsigned::<u8>()? != MAP_KEY_DELIMITER {
            return Err(Error::ExpectedMapKeyDelimiter);
        }
        let value = seed.deserialize(&mut *self.deserializer).map(Some)?;
        if self.deserializer.parse_unsigned::<u8>()? != MAP_KEY_DELIMITER {
            return Err(Error::ExpectedMapKeyDelimiter);
        }
        Ok(value)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        // remove the last MAP_VALUE_DELIMITER and deserialize the value
        if self.deserializer.eat_byte()? != MAP_VALUE_DELIMITER {
            return Err(Error::ExpectedMapValueDelimiter);
        }
        let value = seed.deserialize(&mut *self.deserializer)?;
        if self.deserializer.eat_byte()? != MAP_VALUE_DELIMITER {
            return Err(Error::ExpectedMapValueDelimiter);
        }
        Ok(value)
    }
}
