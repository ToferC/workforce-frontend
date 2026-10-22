use serde::{Deserialize};


// for for single email address
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct EmailForm {
    pub email: String,
}
// form for multiple email addresses
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct EmailsForm {
    pub emails: String,
}