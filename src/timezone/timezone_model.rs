use serde::Serialize;

#[derive(Serialize)]
pub struct Timezone{
    pub name: String,
    pub now: String,
}

