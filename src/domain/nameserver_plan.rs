use super::RegistrarSnapshot;
use facet::Facet;

/// Offline registrar-delegation plan; separate from any hosted-record plan.
#[derive(Facet, Clone, Debug, PartialEq, Eq)]
pub struct NameserverPlan {
    pub before: RegistrarSnapshot,
    pub requested_nameservers: Vec<String>,
}

impl NameserverPlan {
    /// # Errors
    /// Rejects incomplete and malformed requests.
    pub fn build(
        before: RegistrarSnapshot,
        requested_nameservers: Vec<String>,
    ) -> eyre::Result<Self> {
        before.validate()?;
        eyre::ensure!(
            before.provider == "namecheap",
            "first nameserver planner supports Namecheap registrar snapshots only"
        );
        eyre::ensure!(
            requested_nameservers.len() >= 2,
            "at least two nameservers required"
        );
        eyre::ensure!(
            requested_nameservers
                .iter()
                .all(|s| s.contains('.') && !s.chars().any(char::is_whitespace)),
            "invalid nameserver"
        );
        eyre::ensure!(
            requested_nameservers != before.nameservers,
            "nameservers already match"
        );
        Ok(Self {
            before,
            requested_nameservers,
        })
    }

    #[must_use]
    pub fn matches_current(&self, current: &RegistrarSnapshot) -> bool {
        &self.before == current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegation_plan_binds_registrar_state_independent_of_dns_zone() {
        let before = RegistrarSnapshot {
            domain: "example.test".into(),
            provider: "namecheap".into(),
            nameservers: vec!["ns1.azure.test".into(), "ns2.azure.test".into()],
        };
        let plan = NameserverPlan::build(
            before.clone(),
            vec!["dns1.example.test".into(), "dns2.example.test".into()],
        )
        .unwrap();
        assert!(plan.matches_current(&before));
        let mut raced = before;
        raced.nameservers.swap(0, 1);
        assert!(!plan.matches_current(&raced));
    }
}
