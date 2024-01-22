use color_eyre::eyre;

#[derive(Debug, Clone)]
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

impl TryFrom<&str> for Jid {
    type Error = eyre::ErrReport;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (local_part, mut rest) = if let Some(at) = value.find('@') {
            value.split_at(at)
        } else {
            eyre::bail!("@ not found");
        };

        rest = &rest[1..]; // Skip @

        let (domain_part, rest) = if let Some(slash) = rest.find('/') {
            rest.split_at(slash)
        } else {
            eyre::bail!("/ not found");
        };

        let resource_part = &rest[1..]; // Skip /

        Ok(Jid::new(
            local_part.to_owned(),
            domain_part.to_owned(),
            resource_part.to_owned(),
        ))
    }
}
