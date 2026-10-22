use serde::{Deserialize, Serialize};


#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct LoginForm {
    pub csrf_token: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct RegisterForm {
    user_name: String,
    email: String,
    password: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct VerifyForm {
    code: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct PasswordForm {
    password: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct DeleteForm {
    pub verify: String,
}

#[derive(Deserialize, Default, Debug, Serialize)]
#[serde(default)]
pub struct CapabilityForm {
    pub name: String,
    pub level: String,
}