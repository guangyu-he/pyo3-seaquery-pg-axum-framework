use std::process::Command;

fn main() {
    // Determine which Python to query — respect PYO3_PYTHON if set, otherwise default to "python3".
    let python = std::env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_string());

    let output = Command::new(&python)
        .args(["-c", "import sys; print(sys.base_prefix)"])
        .output()
        .unwrap_or_else(|e| panic!("Failed to run {python}: {e}"));

    let base_prefix = String::from_utf8(output.stdout)
        .expect("invalid utf-8 from python")
        .trim()
        .to_string();

    println!("cargo:rustc-env=PY_BASE_PREFIX={base_prefix}");
    println!("cargo:rerun-if-env-changed=PYO3_PYTHON");
}
