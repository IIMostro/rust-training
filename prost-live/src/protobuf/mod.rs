mod person;

pub use person::*;
pub use crate::protobuf::person::person::PhoneNumber;

impl Person{
    pub fn new(name: impl Into<String>,
               id: i32, email: impl Into<String>,
               phone: impl Into<Vec<PhoneNumber>>) -> Self{
        Person{
            name: name.into(),
            id,
            email: email.into(),
            phones: phone.into(),
            ..Default::default()
        }
    }
}

impl PhoneNumber{
    pub fn new(number: impl Into<String>, phone_type: i32) -> Self{
        PhoneNumber{
            number: number.into(),
            phone_type,
        }
    }
}