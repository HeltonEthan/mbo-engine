use std::collections::HashMap;

use crate::api_internal::book::{Active, Inactive, Queue};

#[derive(Debug, Default)]
pub struct Books {
    queue: HashMap<u32, Queue>,
    active: HashMap<u32, Active>,
    inactive: HashMap<u32, Inactive>,
}

impl Books {
    pub fn new() -> Self {
        Self::default()
    }
}
