use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unyeong::config::UnyeongConfig;

fn bench_config_parse(c: &mut Criterion) {
    let toml_str = r#"
[cluster]
name = "test-cluster"
endpoint = "https://endpoint"
global_sans = ["san1"]

[[profiles]]
name = "profile1"
matching_macs = ["mac1"]
[profiles.network]
endpoint_interface = "eno1"
listen_interfaces = ["eno1"]
ignore_offline = true
[profiles.storage]
os_disk = { disk_type = "ssd" }
data_disks = []
ignore_disks = []

[[nodes]]
fqdn = "node1"
mac = "mac1"
"#;

    c.bench_function("parse_config", |b| b.iter(|| {
        let _: UnyeongConfig = toml::from_str(black_box(toml_str)).unwrap();
    }));
}

criterion_group!(benches, bench_config_parse);
criterion_main!(benches);
