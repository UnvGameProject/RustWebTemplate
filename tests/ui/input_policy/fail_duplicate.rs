use input_policy_macros::Normalize;

mod input {
    pub trait Normalize {
        fn normalize(&mut self);
    }
}

#[derive(Normalize)]
struct Example {
    #[normalize(trim, trim)]
    value: String,
}

fn main() {}