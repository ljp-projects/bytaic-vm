use std::path::PathBuf;
use crate::values::Value;

pub(crate) trait RuntimeError {
    fn raise(&self) -> !;
    fn to_string(&self) -> String;
}

pub(crate) struct DeallocatedError(pub String);
pub(crate) struct NoValueError(pub String);
pub(crate) struct CannotConstruct<'a>(pub String, pub &'a Value);
pub(crate) struct NotAllowed(pub String);
pub(crate) struct FileError(pub Option<PathBuf>, pub String);

impl RuntimeError for DeallocatedError {
    fn raise(&self) -> ! {
        panic!("{}", self.to_string())
    }
    
    fn to_string(&self) -> String {
        format!("The item '{}' has been deallocated.\nThis has occurred due to the use of pointers to get around Rust's borrow checker.", self.0)
    }
}


impl RuntimeError for NoValueError {
    fn raise(&self) -> ! {
        panic!("{}", self.to_string())
    }

    fn to_string(&self) -> String {
        format!("The item '{}' did not exist, but was required.", self.0)
    }
}

impl<'a> RuntimeError for CannotConstruct<'a> {
    fn raise(&self) -> ! {
        panic!("{}", self.to_string())
    }

    fn to_string(&self) -> String {
        format!("The item '{}' could not be constructed from the bytes {:?}", self.0, self.1)
    }
}


impl RuntimeError for NotAllowed {
    fn raise(&self) -> ! {
        panic!("{}", self.to_string())
    }

    fn to_string(&self) -> String {
        format!("Not allowed: {}", self.0)
    }
}

impl RuntimeError for FileError {
    fn raise(&self) -> ! {
        panic!("{}", self.to_string())
    }

    fn to_string(&self) -> String {
        format!("File error @ {:?}: {}", self.0, self.1)
    }
}
