use cargo_metadata::MetadataCommand;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	let metadata = MetadataCommand::new().no_deps().exec().unwrap();
	let packages = metadata.packages;
	let name = "oxpm_core";
	let package = packages.iter().find(|p| p.name == name).unwrap();
	let version = &package.version;
	println!("cargo:rustc-env=CORE_VERSION={version}");
}
