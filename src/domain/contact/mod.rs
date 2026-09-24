mod create;
mod error;
mod normalize;
mod update;

#[cfg(test)]
mod tests;

pub(crate) use create::CreateContactInput;
pub(crate) use error::ContactError;
pub(crate) use update::UpdateContactInput;
