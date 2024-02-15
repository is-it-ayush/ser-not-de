use serde::{Deserialize, Serialize};

mod deserializer;
mod error;
mod serializer;

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    is_human: bool,
    money: f64,
    languages: Vec<String>,
}

fn main() {
    let person = Person {
        name: "Ayush Gupta".to_string(),
        age: 19,
        is_human: true,
        money: 0.34,
        languages: ["Rust", "TypeScript", "C"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    };

    println!("Original Data: {:?}\n", person);

    let bytes = serializer::to_bytes(&person).unwrap();
    println!(
        "Serialized Data: {:?}\n",
        bytes
            .iter()
            .map(|&i| format!("{:02x}", i))
            .collect::<Vec<String>>()
            .join(" ")
    );

    let deserialized_person = deserializer::from_bytes::<Person>(&bytes).unwrap();
    println!("Deserialized Data: {:?}\n", deserialized_person);

    // let binary: String = bytes
    //     .iter()
    //     .map(|&i| format!("{:08b}", i))
    //     .collect::<Vec<String>>()
    //     .join(" ");

    // println!("Binary Stream: {}", binary);

    // let hex: String = bytes
    //     .iter()
    //     .map(|&i| format!("{:02x}", i))
    //     .collect::<Vec<String>>()
    //     .join(" ");

    // println!("Hex Stream: {}", hex);
}
