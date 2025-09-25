use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct LoginRequest{
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Token{
    pub exp: i32,
    pub iat: i32,
    pub iss: String,
    pub nbf: i32,
    pub sub: String,
    pub inf: TokenInfo
}
impl Token {
    pub fn build(my_claim: MyClaim) -> Self {

        let inf_map = my_claim.inf.unwrap_or_default();

        let my_claim_inf = TokenInfo {
            uuid: inf_map.get("uuid").cloned().unwrap_or_default().parse::<Uuid>().unwrap(),
            email: inf_map.get("email").cloned().unwrap_or_default(),
            username: inf_map.get("username").cloned().unwrap_or_default(),
        };

        Token {
            exp: my_claim.exp,
            iat: my_claim.iat.unwrap_or_else(|| 0),
            iss: my_claim.iss.unwrap_or_else(|| String::from("")),
            nbf: my_claim.nbf.unwrap_or_else(|| 0),
            sub: my_claim.sub.unwrap_or_else(|| String::from("")),
            inf: my_claim_inf,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TokenInfo{
    pub uuid: Uuid,
    pub email: String,
    pub username: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyClaim {
    pub aud: Option<String>,
    pub exp: i32,
    pub iat: Option<i32>,
    pub iss: Option<String>,
    pub nbf: Option<i32>,
    pub sub: Option<String>,
    pub inf: Option<HashMap<String, String>>,
}

