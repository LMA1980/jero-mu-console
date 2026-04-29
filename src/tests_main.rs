#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_management_ips() {
        // Mock data
        let mut n1_mgmt = HashSet::new();
        let mut n1_data = HashSet::new();
        let ip = "<TEST_IP>".to_string();
        
        // Simulate xeon mgmt
        n1_mgmt.insert(ip.clone());
        n1_data.insert(ip.clone());
        
        // Run logic
        for ip in &n1_mgmt { n1_data.remove(ip); }
        
        assert!(!n1_data.contains(&ip));
        assert!(n1_mgmt.contains(&ip));
    }
}
