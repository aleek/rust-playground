// Copyright 2024 Aleksander Dutkowski
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct UserNotFound;

impl Error for UserNotFound {}

impl fmt::Display for UserNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User not found")
    }
}
