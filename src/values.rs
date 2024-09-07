use crate::errors::CannotConstruct;
use core::slice::SlicePattern;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Rem, Sub};

static TRUE: &[u8] = [1u8].as_slice();
static FALSE: &[u8] = [0u8].as_slice();

#[derive(Debug, Clone)]
pub struct Value {
    pub data_len: u16,
    pub data: Box<[u8]>,
}

impl Value {
    fn new(data: Box<[u8]>) -> Self {
        Value {
            data_len: data.len() as u16,
            data,
        }
    }

    fn new_again(data: [u8; 8]) -> Self {
        Value {
            data_len: data.len() as u16,
            data: Box::from(data),
        }
    }

    pub(crate) fn from_specific<'a, T>(specific: &'a dyn SpecificValue<Value = T>) -> Self {
        specific.to_value()
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.data_len.to_be_bytes().to_vec();

        for byte in &self.data {
            bytes.push(*byte)
        }

        bytes
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let len = u16::from_be_bytes(
            bytes[0..=1]
                .try_into()
                .expect("Failed to convert bytes into u16."),
        );

        let data = &bytes[2usize..(len as usize) + 2];

        Value::new(Box::try_from(data.to_vec()).unwrap())
    }

    pub(crate) fn as_tea_number(&self) -> Result<TeaNumber, CannotConstruct> {
        if let Ok(bytes) = (*self.data).try_into() {
            let value = f64::from_be_bytes(bytes);

            Ok(TeaNumber(value))
        } else {
            Err(CannotConstruct("values::TeaNumber::value".to_string(), self))
        }
    }

    pub(crate) fn as_tea_string(&self) -> Result<TeaStr, CannotConstruct> {
        if let Ok(value) = String::from_utf8(self.data.to_vec()) {
            Ok(TeaStr(value))
        } else {
            Err(CannotConstruct("values::TeaStr::value".to_string(), self))
        }
    }

    pub(crate) fn as_tea_bool(&self) -> TeaBool {
        let value = self.data[0] == 1u8;

        TeaBool(value)
    }

    pub(crate) fn as_tea_null(&self) -> TeaNull {
        TeaNull
    }

    pub(crate) fn as_tea_function(&self) -> Result<TeaFunction, CannotConstruct> {
        if let Ok(bytes) = self.data[1..=2].try_into() {
            let code_len = u16::from_be_bytes(bytes);
            let code = &self.data[3..=(2 + code_len) as usize];

            Ok(TeaFunction::new(code_len, code))
        } else {
            Err(CannotConstruct("values::TeaFunction::code_len".to_string(), self))
        }
    }

    pub(crate) fn as_tea_object(&self) -> Result<TeaObject, CannotConstruct> {
        if let Ok(bytes) = self.data[0..=1].try_into() {
            let num_entries = u16::from_be_bytes(bytes);

            let mut entries: HashMap<String, (Value, u8)> = HashMap::new();
            let mut offset = 2usize;

            loop {
                if entries.len() as u16 >= num_entries {
                    break;
                }

                if let Some(key_len) = self.data.get(offset + 1).map(|v| *v as usize) {
                    offset += 2;

                    if let Ok(key) = String::from_utf8(self.data[offset..=offset + key_len - 1].to_vec()) {
                        offset += key_len;
                        
                        if let Some(value_bytes) = self.data.split_at_checked(offset) {
                            let value = Value::from_bytes(value_bytes.1);

                            offset += (value.data_len as usize) + 2;

                            if let Some(flags) = self.data.get(offset) {
                                offset += 1;

                                entries.insert(key, (value, *flags));
                            } else {
                                return Err(CannotConstruct(format!("values::TeaObject::entry#{}::flags", entries.len()), self))
                            }
                        } else {
                            return Err(CannotConstruct(format!("values::TeaObject::entry#{}::value", entries.len()), self))
                        }
                    } else {
                        return Err(CannotConstruct(format!("values::TeaObject::entry#{}::key", entries.len()), self))
                    }
                } else {
                    return Err(CannotConstruct(format!("values::TeaObject::entry#{}::key_len", entries.len()), self))
                }
            }

            Ok(TeaObject::new(entries))
        } else {
            Err(CannotConstruct("values::TeaObject::num_entries".to_string(), self))
        }
    }
}

pub trait SpecificValue {
    type Value;

    fn value(self) -> Self::Value;
    fn to_value(&self) -> Value;
}

#[derive(Debug)]
pub struct TeaStr(pub String);
pub struct TeaBool(pub bool);
#[derive(Debug)]
pub struct TeaNumber(pub f64);
pub struct TeaNull;

impl SpecificValue for TeaStr {
    type Value = String;

    fn value(self) -> String {
        self.0
    }

    fn to_value(&self) -> Value {
        Value::new(Box::from(self.0.as_bytes()))
    }
}

impl SpecificValue for TeaBool {
    type Value = bool;

    fn value(self) -> bool {
        self.0
    }

    fn to_value(&self) -> Value {
        if self.0 {
            Value::new(Box::from(TRUE))
        } else {
            Value::new(Box::from(FALSE))
        }
    }
}

impl SpecificValue for TeaNumber {
    type Value = f64;

    fn value(self) -> f64 {
        self.0
    }

    fn to_value(&self) -> Value {
        Value::new_again(self.0.to_be_bytes())
    }
}

impl SpecificValue for TeaNull {
    type Value = ();

    fn value(self) -> () {}
    fn to_value(&self) -> Value {
        Value::new(Box::from(FALSE))
    }
}

impl Sub for TeaNumber {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Add for TeaNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Mul for TeaNumber {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for TeaNumber {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Rem for TeaNumber {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl TeaNumber {
    pub(crate) fn pow(self, rhs: Self) -> Self {
        Self(self.0.powf(rhs.0))
    }
}

#[derive(Debug)]
pub struct TeaFunction {
    pub code_len: u16,
    pub code: Box<[u8]>,
}

impl TeaFunction {
    fn new(code_len: u16, code: &[u8]) -> Self {
        TeaFunction {
            code_len,
            code: Box::from(code),
        }
    }
}

impl SpecificValue for TeaFunction {
    type Value = Box<[u8]>;

    fn value(self) -> Self::Value {
        self.code
    }

    fn to_value(&self) -> Value {
        let binding = self.code_len.to_be_bytes();
        let code_len = binding.as_slice();

        let code = self.code.as_slice();

        
        let binding = [code_len, code].concat();

        let bytes = binding.as_slice();

        Value::new(Box::from(bytes))
    }
}

#[derive(Debug)]
pub struct TeaObject {
    pub entries: HashMap<String, (Value, u8)>,
}

impl TeaObject {
    pub(crate) fn new(entries: HashMap<String, (Value, u8)>) -> Self {
        TeaObject { entries }
    }
}

impl SpecificValue for TeaObject {
    type Value = HashMap<String, (Value, u8)>;

    fn value(self) -> Self::Value {
        self.entries
    }

    fn to_value(&self) -> Value {
        let mut bytes: Vec<u8> = vec![];

        let binding = (self.entries.len() as u16).to_be_bytes();
        let entries_size = binding.as_slice();

        for entry in self.entries.clone() {
            let binding = (entry.0.len() as u16).to_be_bytes();
            let key_len = binding.as_slice();

            let entry = [
                key_len,
                entry.0.as_bytes(),
                entry.1 .0.to_bytes().as_slice(),
                &[entry.1 .1],
            ]
            .concat();

            for data in entry {
                bytes.push(data)
            }
        }

        let complete_bytes = [entries_size, bytes.as_slice()].concat();

        Value::new(Box::from(complete_bytes.as_slice()))
    }
}
