use jwt_lib::components::key::{JwtPath, Jwtkey};
use once_cell::sync::Lazy;

pub struct AuthConfig {
    algorithm: String,
    aud_claims: String,
    exp_claims_additional_sec: u32,
    iss_claims: String,
    public_key_path: String,
    private_key_path: String,
}

impl AuthConfig {
    pub fn init_auth_config() -> AuthConfig {
        AuthConfig {
            algorithm: std::env::var("ALGORITHM")
                .expect("ALGORITHM must be specified"),

            aud_claims: std::env::var("AUD_CLAIMS")
                .expect("AUD_CLAIMS must be specified"),

            exp_claims_additional_sec: std::env::var("EXP_CLAIMS_ADDITIONAL_SEC")
                .expect("EXP_CLAIMS_ADDITIONAL_SEC must be specified")
                .parse()
                .expect("EXP_CLAIMS_ADDITIONAL_SEC must be a number"),

            iss_claims: std::env::var("ISS_CLAIMS")
                .expect("ISS_CLAIMS must be specified"),

            public_key_path: std::env::var("PUBLIC_KEY_PATH")
                .expect("PUBLIC_KEY_PATH must be specified"),

            private_key_path: std::env::var("PRIVATE_KEY_PATH")
                .expect("PRIVATE_KEY_PATH must be specified"),
        }
    }

    pub fn get_algorithm() -> &'static str {
        &AUTH_CONFIG.algorithm
    }

    pub fn get_aud_claims() -> String {
        AUTH_CONFIG.aud_claims.to_string()
    }

    pub fn get_exp_claims_additional_sec() -> usize {
        AUTH_CONFIG.exp_claims_additional_sec as usize
    }

    pub fn get_iss_claims() -> String {
        AUTH_CONFIG.iss_claims.to_string()
    }

    pub fn get_public_key_path() -> &'static str {
        &AUTH_CONFIG.public_key_path
    }

    pub fn get_private_key_path() -> &'static str {
        &AUTH_CONFIG.private_key_path
    }

    pub async fn set_auth_keys(private_key_path: &str, public_key_path: &str) {

        match JwtPath::set_private_key_path(private_key_path){
            Ok(_) => (),
            Err(e) => panic!("Set private key path error: {}", &e),
        };

        match JwtPath::set_public_key_path(public_key_path){
            Ok(_) => (),
            Err(e) => panic!("Set public key path error: {}", &e)
        };

        match Jwtkey::set_private_key(){
            Ok(_) => (),
            Err(e) => panic!("Set private key error: {}", &e)
        };

        match Jwtkey::set_public_key(){
            Ok(_) => (),
            Err(e) => panic!("Set public key error: {}", &e)
        }
    }
}

static AUTH_CONFIG: Lazy<AuthConfig> = Lazy::new(AuthConfig::init_auth_config);