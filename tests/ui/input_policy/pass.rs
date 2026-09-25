use input_policy_macros::Normalize;

mod input {
    pub trait Normalize {
        fn normalize(&mut self);
    }
}

use input::Normalize as _;

#[derive(Normalize)]
struct Example {
    #[normalize(trim)]
    name: String,

    #[normalize(trim, ascii_lowercase)]
    email: String,
}

fn main() {
    let mut value = Example {
        name: "  José Álvarez  ".to_owned(),
        email: "  ADA@EXAMPLE.COM  ".to_owned(),
    };

    value.normalize();

    assert_eq!(value.name, "José Álvarez");
    assert_eq!(value.email, "ada@example.com");
}