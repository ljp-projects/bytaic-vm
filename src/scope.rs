use crate::errors::{DeallocatedError, FileError, NoValueError, RuntimeError};
use crate::values::{SpecificValue, TeaNumber, TeaObject, Value};
use std::collections::{HashMap, VecDeque};
use std::f64::consts::{PI, TAU};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Scope {
    stack: VecDeque<Value>,
    vars: HashMap<u32, Value>,
    parent: Option<*mut Scope>,
    file_descriptors: HashMap<u32, PathBuf>
}

impl Scope {
    pub(crate) fn new(parent: Option<*mut Scope>) -> Self {
        Scope {
            stack: VecDeque::new(),
            vars: HashMap::new(),
            parent,
            file_descriptors: HashMap::new()
        }
    }

    pub(crate) fn new_global() -> Self {
        let mut s = Scope {
            stack: VecDeque::new(),
            vars: HashMap::new(),
            parent: None,
            file_descriptors: HashMap::new()
        };

        let math: HashMap<String, (Value, u8)> = HashMap::from([
            ("pi".into(), (Value::from_specific(&TeaNumber(PI)), 2u8)),
            ("tau".into(), (Value::from_specific(&TeaNumber(TAU)), 2u8)),
        ]);

        let io: HashMap<String, (Value, u8)> = HashMap::from([
            ("stdout".into(), (Value::from_specific(&TeaNumber(0.)), 2u8)),
            ("stdin".into(), (Value::from_specific(&TeaNumber(1.)), 2u8)),
        ]);
        
        s.set_var(&0, &TeaObject::new(io).to_value());
        s.set_var(&1, &TeaObject::new(math).to_value());

        s
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop_back()
    }

    pub fn push(&mut self, value: &Value) {
        self.stack.push_back(value.clone())
    }

    pub fn get_var(&self, idx: &u32) -> Option<Value> {
        if let Some(var) = self.vars.get(&idx) {
            Option::from(var.clone())
        } else {
            unsafe { self.parent?.as_ref() }?.get_var(idx)
        }
    }
    
    pub fn add_fd(&mut self, fd: &u32, path: &PathBuf) {
        self.file_descriptors.insert(fd.clone(), path.clone());
    }
    
    pub fn read_fd(&self, fd: &u32) -> Result<&[u8], FileError> {
        if let Some(path) = self.file_descriptors.get(fd) {
            if let Ok(mut file) = File::create_new(path) {
                let vec: *mut Vec<u8> = &mut vec![];
                
                if let Ok(_) = file.read_to_end(unsafe { vec.as_mut() }.unwrap()) {
                    Ok(unsafe { vec.as_ref() }.unwrap().as_slice())
                } else {
                    Err(FileError(Some(path.clone()), "could not read file".to_string()))
                }
            } else {
                Err(FileError(Some(path.clone()), "could not open file".to_string()))
            }
        } else {
            Err(FileError(None, format!("could not find file at fd {fd}")))
        }
    }

    pub fn write_fd(&self, fd: &u32, content: &[u8]) -> Result<(), FileError> {
        if let Some(path) = self.file_descriptors.get(fd) {
            if let Ok(mut file) = File::create_new(path) {
                file.write_all(content).map_err(|_| FileError(Some(path.clone()), "could not write to file".to_string()))
            } else {
                Err(FileError(Some(path.clone()), "could not open file".to_string()))
            }
        } else {
            Err(FileError(None, format!("could not find file at fd {fd}")))
        }
    }

    pub fn set_var(&mut self, idx: &u32, value: &Value) -> () {
        self.vars.insert(*idx, value.clone());

        if let Some(parent) = self.parent {
            unsafe { parent.as_mut() }
                .unwrap_or_else(|| DeallocatedError("Scope::parent".into()).raise())
                .set_var(idx, value);

            self.vars.remove(idx);
        }
    }
}
