use assert_cmd::Command;

fn get_bin() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn it_prints_version_info_successfully() {
    get_bin()
        .arg("-v")
        .assert()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")))
        .success();
}

#[test]
fn it_has_special_char_warning() {
    get_bin()
        .arg("filename~")
        .assert()
        .stdout(predicates::str::contains("Warning:"))
        .failure();
    get_bin()
        .arg("~/newfile.csv")
        .assert()
        .stdout(predicates::str::contains("Warning:"))
        .failure();
    get_bin()
        .arg("$HOME")
        .assert()
        .stdout(predicates::str::contains("Warning:"))
        .failure();
}