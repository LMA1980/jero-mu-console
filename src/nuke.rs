use std::process::Command;
use anyhow::Result;

pub fn run(purge: bool) -> Result<()> {
    println!("💣 NUKING CLUSTER...");
    let targets = "<CLUSTER_IP_LIST>";
    let mut args = vec!["reset", "--nodes", targets, "--reboot", "--insecure", "--wait=false"];
    if purge {
        println!("🔥 Purging all persistent data disks...");
        args.push("--wipe");
    }
    Command::new("talosctl").args(args).status().ok(); 
    Command::new("talosctl").args(["config", "remove", "jero-mu"]).status().ok();

    use std::fs;
    use std::path::Path;
    println!("🧹 Cleaning local workspace...");
    let talos_dir = Path::new("talos");
    if talos_dir.exists() {
        for entry in fs::read_dir(talos_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let n = name.to_str().unwrap_or("");
            if n != "node-1.yaml" && n != "node-2.yaml" {
                if entry.path().is_dir() { fs::remove_dir_all(entry.path())?; }
                else { fs::remove_file(entry.path())?; }
            }
        }
    }
    if Path::new(".env").exists() { fs::remove_file(".env")?; }
    Ok(())
}
