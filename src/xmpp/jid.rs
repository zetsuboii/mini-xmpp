#[derive(Debug)]
pub struct Jid {
    pub local_part: String,
    pub domain_part: String,
    pub resource_part: String,
}

impl Jid {
    pub fn new(local_part: String, domain_part: String, resource_part: String) -> Self {
        Self {
            local_part,
            domain_part,
            resource_part,
        }
    }

    pub fn address(&self) -> String {
        format!("{}@{}", self.local_part, self.domain_part)
    }

    pub fn resource_part(&self) -> &str {
        self.resource_part.as_ref()
    }
}

impl ToString for Jid {
    fn to_string(&self) -> String {
        format!(
            "{}@{}/{}",
            self.local_part, self.domain_part, self.resource_part
        )
    }
}
