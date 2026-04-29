use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum NodeState {
    Discovered,
    Maintenance,
    ManagedPending,
    Healthy,
    Unhealthy,
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct LinkStatus { pub metadata: LinkMetadata, pub spec: LinkSpec }
#[derive(Debug, Deserialize)]
pub struct LinkMetadata { pub id: String }
#[derive(Debug, Deserialize)]
pub struct LinkSpec { #[serde(rename = "hardwareAddr")] pub hardware_addr: String }

#[derive(Debug, Deserialize)]
pub struct DeviceStatus { pub metadata: DeviceMetadata, pub spec: DeviceSpec }
#[derive(Debug, Deserialize)]
pub struct DeviceMetadata { pub id: String }
#[derive(Debug, Deserialize)]
pub struct DeviceSpec { pub model: String }
