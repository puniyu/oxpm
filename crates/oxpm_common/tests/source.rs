use oxpm_common::SourceType;

#[test]
fn parse_registry() {
	let source = SourceType::parse("registry+https://registry.npmjs.org");
	assert!(matches!(source, SourceType::Registry(_)));
	let inner = match source { SourceType::Registry(s) => s, _ => unreachable!() };
	assert_eq!(inner.as_str(), "https://registry.npmjs.org");
}

#[test]
fn parse_git() {
	let source = SourceType::parse("git+https://github.com/user/repo#abc");
	assert!(matches!(source, SourceType::Git(_)));
}

#[test]
fn parse_file() {
	let source = SourceType::parse("file:../packages/local");
	assert!(matches!(source, SourceType::File(_)));
}

#[test]
fn parse_link() {
	let source = SourceType::parse("link:../packages/local");
	assert!(matches!(source, SourceType::Link(_)));
}

#[test]
fn parse_workspace() {
	let source = SourceType::parse("workspace:packages/my-lib");
	assert!(matches!(source, SourceType::Workspace(_)));
}

#[test]
fn parse_tarball() {
	let source = SourceType::parse("tarball:./packages/my-pkg-1.0.0.tgz");
	assert!(matches!(source, SourceType::Tarball(_)));
	let inner = match source { SourceType::Tarball(s) => s, _ => unreachable!() };
	assert_eq!(inner.path(), "./packages/my-pkg-1.0.0.tgz");
}

#[test]
fn to_source_string() {
	let source = SourceType::parse("registry+https://registry.npmjs.org");
	assert_eq!(source.to_source_string().as_str(), "registry+https://registry.npmjs.org");
}

#[test]
fn is_registry() {
	assert!(SourceType::parse("registry+https://npmjs.org").is_registry());
	assert!(!SourceType::parse("git+https://github.com/user/repo").is_registry());
}

#[test]
fn is_local() {
	assert!(SourceType::parse("file:./local").is_local());
	assert!(SourceType::parse("link:./local").is_local());
	assert!(SourceType::parse("workspace:packages/a").is_local());
	assert!(SourceType::parse("tarball:./local.tgz").is_local());
	assert!(!SourceType::parse("registry+https://npmjs.org").is_local());
}

#[test]
fn is_tarball() {
	assert!(SourceType::parse("tarball:./pkg.tgz").is_tarball());
	assert!(!SourceType::parse("file:./local").is_tarball());
}

#[test]
fn tarball_to_source_string() {
	let source = SourceType::parse("tarball:./packages/my-pkg-1.0.0.tgz");
	assert_eq!(source.to_source_string().as_str(), "tarball:./packages/my-pkg-1.0.0.tgz");
}

#[test]
fn tarball_as_tarball() {
	let source = SourceType::parse("tarball:./pkg.tgz");
	let inner = source.as_tarball().unwrap();
	assert_eq!(inner.path(), "./pkg.tgz");
}

#[test]
fn try_from() {
	let source = SourceType::try_from("registry+https://npmjs.org").unwrap();
	assert!(matches!(source, SourceType::Registry(_)));
}

#[test]
fn try_from_empty() {
	let result = SourceType::try_from("");
	assert!(result.is_err());
}

#[test]
fn display() {
	let source = SourceType::parse("registry+https://npmjs.org");
	assert_eq!(format!("{}", source), "registry+https://npmjs.org");
}