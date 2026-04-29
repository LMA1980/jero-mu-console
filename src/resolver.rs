use crate::{LinkStatus, DeviceStatus};
use crate::config::StorageRule;
use anyhow::{Result, anyhow};

pub struct HardwareResolver;

impl HardwareResolver {
    pub fn resolve_nic(links: &[LinkStatus], rule: &str) -> Result<String> {
        links.iter()
            .find(|l| l.metadata.id == rule)
            .map(|l| l.metadata.id.clone())
            .ok_or_else(|| anyhow!("Could not resolve NIC with rule: {}", rule))
    }

    pub fn resolve_storage(devices: &[DeviceStatus], rule: &StorageRule) -> Result<String> {
        if let Some(path) = &rule.path {
            return Ok(path.clone());
        }

        let filtered: Vec<&DeviceStatus> = devices.iter()
            .filter(|d| {
                if let Some(disk_type) = &rule.disk_type {
                    let model = d.spec.model.to_lowercase();
                    match disk_type.as_str() {
                        "nvme" => model.contains("nvme"),
                        "ssd" => model.contains("ssd") || model.contains("flash"),
                        "hdd" => !model.contains("nvme") && !model.contains("ssd"),
                        _ => true,
                    }
                } else {
                    true
                }
            })
            .collect();

        if let Some(index) = rule.index {
            filtered.get(index)
                .map(|d| format!("/dev/{}", d.metadata.id))
                .ok_or_else(|| anyhow!("Could not find disk at index {} with type {:?}", index, rule.disk_type))
        } else {
            filtered.first()
                .map(|d| format!("/dev/{}", d.metadata.id))
                .ok_or_else(|| anyhow!("No disks found matching rule: {:?}", rule))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LinkMetadata, LinkSpec, DeviceMetadata, DeviceSpec};

    #[test]
    fn test_resolve_nic_success() {
        let links = vec![
            LinkStatus {
                metadata: LinkMetadata { id: "eno1".into() },
                spec: LinkSpec { hardware_addr: "aa:bb:cc".into() },
            },
        ];
        let res = HardwareResolver::resolve_nic(&links, "eno1").unwrap();
        assert_eq!(res, "eno1");
    }

    #[test]
    fn test_resolve_nic_fail() {
        let links = vec![];
        let res = HardwareResolver::resolve_nic(&links, "eno1");
        assert!(res.is_err());
    }

    #[test]
    fn test_resolve_storage_by_path() {
        let devices = vec![];
        let rule = StorageRule { path: Some("/dev/sda".into()), disk_type: None, min_size: None, index: None };
        let res = HardwareResolver::resolve_storage(&devices, &rule).unwrap();
        assert_eq!(res, "/dev/sda");
    }

    #[test]
    fn test_resolve_storage_by_type_and_index() {
        let devices = vec![
            DeviceStatus {
                metadata: DeviceMetadata { id: "nvme0n1".into() },
                spec: DeviceSpec { model: "Samsung NVMe".into() },
            },
            DeviceStatus {
                metadata: DeviceMetadata { id: "sda".into() },
                spec: DeviceSpec { model: "SATA SSD".into() },
            },
        ];
        
        let rule_nvme = StorageRule { path: None, disk_type: Some("nvme".into()), min_size: None, index: Some(0) };
        let res_nvme = HardwareResolver::resolve_storage(&devices, &rule_nvme).unwrap();
        assert_eq!(res_nvme, "/dev/nvme0n1");

        let rule_ssd = StorageRule { path: None, disk_type: Some("ssd".into()), min_size: None, index: Some(0) };
        let res_ssd = HardwareResolver::resolve_storage(&devices, &rule_ssd).unwrap();
        assert_eq!(res_ssd, "/dev/sda");
    }
}
