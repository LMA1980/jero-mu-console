use std::fs;
use anyhow::Result;

pub struct PkiHydrator;

impl PkiHydrator {
    pub fn get_mu_root_ca() -> Result<String> {
        let paths = [
            "infra/pki/mu-root-ca.crt",
            "infra/pki/mu-root-ca-secret/tls.crt",
        ];
        
        for path in paths {
            if let Ok(content) = fs::read_to_string(path) {
                return Ok(content);
            }
        }
        
        Err(anyhow::anyhow!("Could not find Mu Root CA in expected paths"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mu_root_ca_failure() {
        let result = PkiHydrator::get_mu_root_ca();
        assert!(result.is_err());
    }
}
