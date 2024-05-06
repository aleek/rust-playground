// Copyright 2024 Aleksander Dutkowski
pub mod usernotfound;
use std::fmt;
use crate::mysqldb::usernotfound::UserNotFound;

pub enum UserSex {
    Male,
    Female,
    Other,
}

pub struct User {
    pub id: u32,
    pub name: String,
    pub sex: UserSex,
}

pub trait PersistentStorage {
    fn get_all_users(&self) -> Vec<User>;
    fn get_user_by_id(&self, id: u32) -> Result<User, UserNotFound>;
}

pub struct MysqlDatabase {
    handle: u32,
}

pub fn new_mysql_db(handle: u32) -> Box<dyn PersistentStorage> {
    return Box::new(MysqlDatabase{
        handle: handle,
    })
}

impl PersistentStorage for MysqlDatabase {
    fn get_all_users(&self) -> Vec<User> {
        let mut v: Vec<User> = Vec::new();

        v.push(User{
            id: 1,
            name: "Aleksander".into(),
            sex: UserSex::Male,
        });

        return v;
    }

    fn get_user_by_id(&self, id: u32) -> Result<User, UserNotFound> {
        if id != 1 {
            return Err(UserNotFound);
        }

        Ok(User { id: 1, name: "Aleksander".into(), sex: UserSex::Male })
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserName {:?}", self.name)
    }
}