use std::process::Command;
fn main() {
    let output = Command::new("nmap").args(["-p", "50000", "-n", "<MGMT_NETWORK_CIDR>", "--exclude", "<GATEWAY_IP>", "--open", "-oG", "-"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let ips: Vec<String> = stdout.lines()
        .filter(|l| l.contains("Host:"))
        .filter_map(|l| l.split_whitespace().nth(1))
        .map(|s| s.to_string())
        .collect();
    println!("Total IPs: {}", ips.len());
    println!("IPs: {:?}", ips);
}
