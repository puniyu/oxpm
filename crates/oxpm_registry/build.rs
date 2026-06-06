fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	let version = oxpm_version::VERSION.to_string();
	println!("cargo:rustc-env=VERSION={version}");
}
