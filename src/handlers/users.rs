// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use serde::{Deserialize};


#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct UserForm {
    user_name: String,
    email: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct AdminUserForm {
    user_name: String,
    email: String,
    role: String,
    validated: String,
}

