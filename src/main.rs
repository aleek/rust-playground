use crate::{calc::new_calc, mysqldb::new_mysql_db};

// Copyright 2024 Aleksander Dutkowski
pub mod calc;
pub mod mysqldb;

fn main() {
    println!("Hello, world!");

    let mut c: calc::Calc = new_calc();

    c.set_first_argument(5);
    c.set_second_argument(4);

    let x: u32 = c.sum();

    println!("Sum of 5 and 4 is {x}");

    let mut db = new_mysql_db(15);

    let result = db.get_user_by_id(1);

    let file = match result {
        Ok(user) => user,
        Err(error) => panic!("Problem: {:?}", error)
    };

    println!("User {file}");
}
