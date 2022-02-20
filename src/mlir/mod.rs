pub mod raw {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use raw::MlirStringRef;

impl From<&str> for MlirStringRef {
    fn from(item: &str) -> Self {
        MlirStringRef {
            data: item.as_ptr() as *const _,
            length: item.len() as u64,
        }
    }
}

pub mod attribute;
pub mod block;
pub mod context;
pub mod operation;
pub mod region;
pub mod value;
