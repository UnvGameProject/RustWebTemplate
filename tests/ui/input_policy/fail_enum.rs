use input_policy_macros::Normalize;

mod input {
    pub trait Normalize {
        fn normalize(&mut self);
    }
}

#[derive(Normalize)]
enum Example {
    Value,
}

fn main() {}