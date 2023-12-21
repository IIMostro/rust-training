
// pub mod items{
//     include!(concat!(env!("OUT_DIR"), "/person.rs"));
// }
mod protobuf;
use prost::Message;
use protobuf::*;


fn main() {
    let phones = vec![PhoneNumber::new("1234567890", 0)];
    let person = Person::new("John Doe", 123, "neptune@gmail.com", phones);
    println!("person: {:?}", &person);
    let v1 = person.encode_to_vec();
    let v2 = person.encode_length_delimited_to_vec();

    let person2 = Person::decode(&v1[..]).unwrap();
    let json_value = serde_json::to_string_pretty(&person2).unwrap();
    println!("v1: {:?}, \nv2:{:?}", v1, v2);
    println!("json: {}", json_value);
}
