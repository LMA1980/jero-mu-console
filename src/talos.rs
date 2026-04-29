pub fn talos_reset() -> Result<std::process::Child, std::io::Error> {
    std::process::Command::new("talosctl")
        .arg("reset")
        .spawn()
}