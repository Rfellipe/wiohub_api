use std::process::Command;

pub fn send_to_zabbix(metric: &str, value: String) {
    let hostname = "api_rust"; // Change this to match your Zabbix hostname
    let zabbix_server = "192.168.122.116"; // Replace with your Zabbix server IP

    // verify that the zabbix_sender is installed
    if !Command::new("which")
        .arg("zabbix_sender")
        .output()
        .expect("Failed to check if zabbix_sender is installed")
        .status
        .success()
    {
        eprintln!("zabbix_sender is not installed");
        return;
    }

    let output = Command::new("zabbix_sender")
        .args(&[
            "-z",
            zabbix_server,
            "-s",
            hostname,
            "-k",
            metric,
            "-o",
            &value.to_string(),
        ])
        .output()
        .expect("Failed to send data to Zabbix");

    if !output.status.success() {
        eprintln!("Zabbix Sender failed: {:?}", output);
    }
}
