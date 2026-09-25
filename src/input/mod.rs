pub(crate) use input_policy_macros::Normalize;

pub(crate) mod validation;

pub(crate) trait Normalize {
    fn normalize(&mut self);
}
