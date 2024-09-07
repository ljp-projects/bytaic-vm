#![feature(slice_pattern)]

extern crate core;

use crate::errors::{DeallocatedError, NoValueError, NotAllowed, RuntimeError};
use crate::scope::Scope;
use crate::values::{SpecificValue, TeaBool, TeaNumber, TeaStr, Value};
use std::{fs, io::stdin};
use std::collections::HashMap;

mod errors;
mod opcodes;
mod scope;
mod values;

fn combine_u8_to_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    if offset >= bytes.len() || offset + 4 > bytes.len() {
        return None;
    }

    Some(u32::from_be_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("Failed to convert bytes to u32"),
    ))
}

fn main() {
    let bytes = fs::read("src/test.bin").unwrap();

    run(&bytes, &mut Scope::new_global());
}

fn run(bytes: &Vec<u8>, scope: *mut Scope) {
    let mut pc = 0usize;
    let mut labels = HashMap::new();

    while pc < bytes.len() {
        unsafe {
            let opcode = bytes[pc];

            match opcode {
                opcodes::PUSH => {
                    pc += 1;

                    let value = Value::from_bytes(bytes.split_at(pc).1);

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&value);

                    pc += value.data_len.clone() as usize + 2;
                }

                opcodes::STORE => {
                    pc += 1;

                    let value = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("Store::stack::value".into()).raise());

                    let values = &bytes[pc..pc + 4];

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .set_var(&combine_u8_to_u32(values, 0).unwrap(), &value);

                    pc += 4;
                }

                opcodes::LOAD => {
                    pc += 1;

                    let values = &bytes[pc..pc + 4];

                    if let Some(idx) = combine_u8_to_u32(values, 0) {
                        let value = scope
                            .as_mut()
                            .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                            .get_var(&idx)
                            .clone()
                            .unwrap_or_else(|| {
                                NoValueError(format!("Load::variable(idx = {})", idx)).raise()
                            });

                        scope
                            .as_mut()
                            .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                            .push(&value)
                    }

                    pc += 4;
                }

                opcodes::NADD => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NADD::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NADD::stack::left".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&Value::from_specific(&TeaNumber(
                            left.value() + right.value(),
                        )))
                }

                opcodes::NSUB => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NSUB::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NSUB::stack::left".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&(left - right).to_value())
                }

                opcodes::NMUL => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NMUL::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NMUL::stack::left".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&(left * right).to_value())
                }

                opcodes::NDIV => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NDIV::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NDIV::stack::left".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&(left / right).to_value())
                }

                opcodes::NMOD => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NMOD::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NMOD::stack::left".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&(left % right).to_value())
                }

                opcodes::NPOW => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NPOW::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NPOW::stack::left".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&left.pow(right).to_value())
                }

                opcodes::SMUL => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("SMUL::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("SMUL::stack::left".to_string()).raise())
                        .as_tea_string()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&Value::from_specific(&TeaStr(
                            left.value().repeat(right.value() as usize),
                        )))
                }

                opcodes::CALL => {
                    pc += 1;

                    let f = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("CALL::stack::fn".to_string()).raise())
                        .as_tea_function()
                        .unwrap_or_else(|e| e.raise());

                    run(
                        &f.code.as_ref().to_vec(),
                        &mut Scope::new(Option::from(scope)).clone().clone(),
                    )
                }

                opcodes::GET => {
                    pc += 1;

                    let obj = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("GET::stack::obj".to_string()).raise())
                        .as_tea_object()
                        .unwrap_or_else(|e| e.raise());

                    let key_len = bytes[pc];

                    pc += 1;

                    let key = String::from_utf8(bytes[pc..=pc + (key_len - 1) as usize].to_vec())
                        .unwrap();

                    pc += key_len as usize;

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(
                            &obj.entries
                                .get(&key)
                                .unwrap_or_else(|| {
                                    NoValueError(format!("GET::stack::Object::{key}")).raise()
                                })
                                .to_owned()
                                .0,
                        );
                }

                opcodes::WRITE => {
                    pc += 1;

                    let value = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("WRITE::stack::value".to_string()).raise())
                        .as_tea_string()
                        .unwrap_or_else(|e| e.raise());

                    let fd = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("WRITE::stack::fd".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise())
                        .0 as u32;

                    match fd {
                        0 => {
                            print!("{}", value.value())
                        }

                        1 => NotAllowed("writing to stdin".to_string()).raise(),

                        fd => {
                            unsafe { scope.as_mut() }
                                .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                                .write_fd(&fd, value.0.as_bytes())
                                .unwrap_or_else(|e| e.raise());
                        }
                    }
                }

                opcodes::READLN => {
                    pc += 1;

                    let fd = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("READ::stack::fd".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise())
                        .0 as u32;

                    match fd {
                        0 => NotAllowed("reading from stdout".to_string()).raise(),

                        1 => {
                            let str: *mut String = &mut String::new();

                            if let Ok(_) = stdin().read_line(unsafe { str.as_mut() }.unwrap()) {
                                scope
                                    .as_mut()
                                    .unwrap_or_else(|| {
                                        DeallocatedError("Scope::global".into()).raise()
                                    })
                                    .push(
                                        &TeaStr(unsafe { str.as_ref() }.unwrap().clone())
                                            .to_value(),
                                    )
                            };
                        }

                        fd => {
                            unsafe { scope.as_mut() }
                                .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                                .read_fd(&fd)
                                .unwrap_or_else(|e| e.raise());
                        }
                    }
                }
                
                opcodes::MARKER => {
                    pc += 1;

                    let values = &bytes[pc..pc + 4];
                    let idx = combine_u8_to_u32(values, 0).unwrap();
                    
                    pc += 4;
                    
                    labels.insert(idx, pc);
                }
                
                opcodes::GOTO => {
                    pc += 1;

                    let values = &bytes[pc..pc + 4];
                    let idx = &combine_u8_to_u32(values, 0).unwrap();

                    pc = labels[idx];
                }
                
                opcodes::GOTO_IF => {
                    pc += 1;
                    
                    let cond =
                        unsafe { scope.as_mut() }
                            .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                            .pop()
                            .unwrap_or_else(|| NoValueError("GOTO_IF::stack::cond".to_string()).raise())
                            .as_tea_bool();

                    if cond.value() {
                        let values = &bytes[pc..pc + 4];
                        let idx = &combine_u8_to_u32(values, 0).unwrap();

                        pc = labels[idx];
                    } else {
                        pc += 4;
                    }
                }
                
                opcodes::EQ => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("EQ::stack::right".to_string()).raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("EQ::stack::left".to_string()).raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&TeaBool(left.data == right.data).to_value())
                }

                opcodes::NGT => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NGT::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NGT::stack::left".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&TeaBool(left.value() > right.value()).to_value())
                }

                opcodes::NLT => {
                    pc += 1;

                    let right = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NLT::stack::right".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    let left = scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .pop()
                        .unwrap_or_else(|| NoValueError("NLT::stack::left".to_string()).raise())
                        .as_tea_number()
                        .unwrap_or_else(|e| e.raise());

                    scope
                        .as_mut()
                        .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                        .push(&TeaBool(left.value() < right.value()).to_value())
                }

                opcodes::PRINT => {
                    pc += 1;

                    println!(
                        "{:#?}",
                        scope
                            .as_mut()
                            .unwrap_or_else(|| DeallocatedError("Scope::global".into()).raise())
                            .pop()
                    );
                }

                _ => (),
            }
        }
    }
}
