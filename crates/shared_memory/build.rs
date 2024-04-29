use std::{env, process::Command};

fn execute_success(cmd: &mut Command) {
    let mut pid = cmd.spawn().unwrap();
    let res = pid.wait().unwrap();
    assert!(res.success());
}

fn main() {
    let out = env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=AR");

    println!("cargo:rerun-if-changed=src/shmutils.c");

    println!("cargo:rustc-link-lib=static=shmutils");
    println!("cargo:rustc-link-search=native={}", out);

    let cc = env::var("CC").unwrap_or("cc".into());
    let mut c = Command::new(cc);
    c.arg("./src/shmutils.c")
        .arg("-o")
        .arg(format!("{}/shmutils.o", out))
        .arg("-c");
    execute_success(&mut c);

    let ar = env::var("AR").unwrap_or("ar".into());
    let mut ar = Command::new(ar);
    ar.arg("rcs")
        .arg(format!("{}/libshmutils.a", out))
        .arg(format!("{}/shmutils.o", out));
    execute_success(&mut ar);
}
