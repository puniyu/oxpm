
use std::collections::VecDeque;
use std::path::PathBuf;

use indexmap::IndexMap;
use oxpm_config::{Config, ScopeType};
use oxpm_package_json::PackageJson;
use oxpm_semver::VersionRangeKind;
use smol_str::SmolStr;

use crate::Result;
use crate::cache::PackageCache;
use crate::error::Error;
use crate::types::{DepNode, DepRoot, DepTask, DepType, DependencyTree};

pub struct Resolver {
	config: Config,
	overrides: IndexMap<SmolStr, SmolStr>,
	cache: PackageCache,
	package: Option<PackageJson>,
}

impl Resolver {
	pub fn new() -> Result<Self> {
		Self::with_config(Config::default())
	}

	pub fn with_config(config: Config) -> Result<Self> {
		Ok(Self { config, overrides: IndexMap::new(), cache: PackageCache::new(), package: None })
	}

	/// 设置待解析的 PackageJson
	pub fn with_package_json(mut self, pkg: PackageJson) -> Self {
		self.package = Some(pkg);
		self
	}

	fn get_registry_for_scope(&self, scope: Option<&ScopeType>) -> Result<oxpm_registry::Registry> {
		let url = self.config.registry.for_scope(scope);
		let registry = oxpm_registry::Registry::new(url).map_err(Error::Registry)?;

		if let Some(auth) = self.config.auth.for_scope(scope) {
			return Ok(apply_auth(registry, auth));
		}
		Ok(registry)
	}

	pub async fn resolve(mut self) -> Result<DependencyTree> {
		let pkg = self
			.package
			.take()
			.ok_or_else(|| Error::PackageNotFound(SmolStr::new("<no package>")))?;

		let name = SmolStr::new(
			pkg.name.as_ref().ok_or_else(|| Error::PackageNotFound(SmolStr::new("<anonymous>")))?,
		);
		let version = pkg.version.clone().ok_or_else(|| Error::VersionNotFound {
			name: name.clone(),
			range: SmolStr::new("missing version in package.json"),
		})?;

		self.apply_overrides(&pkg.overrides);

		let root = DepRoot::new(name.clone(), version, PathBuf::from("."));
		let mut tree = DependencyTree::new(root);

		let mut tasks: VecDeque<DepTask> = self.collect_deps(&pkg).into_iter().collect();
		let mut processed = std::collections::HashSet::new();

		while let Some(task) = tasks.pop_front() {
			let key = (task.name.clone(), task.range.canonical());
			if processed.contains(&key) {
				continue;
			}
			processed.insert(key);

			let resolved =
				resolve_one_dep(&self, &task.name, &task.range.canonical(), task.dep_type).await;

			match resolved {
				Ok(node) => {
					tree.nodes.push(node.clone());
				}
				Err(e) => {
					if task.dep_type != DepType::Optional {
						return Err(e);
					}
				}
			}
		}

		Ok(tree)
	}

	/// 收集 PackageJson 中的所有依赖为待处理任务
	fn collect_deps(&self, pkg: &PackageJson) -> Vec<DepTask> {
		let mut tasks = Vec::new();

		let add_deps = |deps: &Option<IndexMap<SmolStr, SmolStr>>,
		                dep_type: DepType,
		                tasks: &mut Vec<DepTask>| {
			if let Some(deps) = deps {
				for (name, range) in deps {
					if let Ok(range_kind) = VersionRangeKind::parse(range.as_str()) {
						tasks.push(DepTask::new(name.clone(), range_kind, dep_type));
					}
				}
			}
		};

		add_deps(&pkg.dependencies, DepType::Production, &mut tasks);
		add_deps(&pkg.dev_dependencies, DepType::Development, &mut tasks);
		add_deps(&pkg.peer_dependencies, DepType::Peer, &mut tasks);
		add_deps(&pkg.optional_dependencies, DepType::Optional, &mut tasks);

		tasks
	}

	/// 应用 overrides
	fn apply_overrides(
		&mut self,
		overrides: &Option<IndexMap<SmolStr, oxpm_package_json::OverrideValue>>,
	) {
		if let Some(overrides) = overrides {
			for (name, value) in overrides {
				match value {
					oxpm_package_json::OverrideValue::Version(v) => {
						self.overrides.insert(name.clone(), v.clone());
					}
					oxpm_package_json::OverrideValue::Nested(nested) => {
						if let Some(override_value) = nested.get(name)
							&& let oxpm_package_json::OverrideValue::Version(v) = override_value
						{
							self.overrides.insert(name.clone(), v.clone());
						}
					}
				}
			}
		}
	}
}

impl Default for Resolver {
	fn default() -> Self {
		Self::new().expect("failed to create default resolver")
	}
}

/// 根据 AuthConfig 应用认证到 Registry
fn apply_auth(
	registry: oxpm_registry::Registry,
	auth: &oxpm_config::RegistryAuthConfig,
) -> oxpm_registry::Registry {
	use oxpm_registry::RegistryAuth;

	if let Some(ref token) = auth.token {
		registry.auth(RegistryAuth::BearerToken(token.clone()))
	} else if let Some(ref auth_str) = auth.auth {
		registry.auth(RegistryAuth::BasicToken(auth_str.clone()))
	} else if auth.username.is_some() || auth.password.is_some() {
		let username = auth.username.clone().unwrap_or_default();
		let password = auth.password.clone().unwrap_or_default();
		registry.auth(RegistryAuth::Basic { username, password })
	} else {
		registry
	}
}

/// 解析单个依赖
async fn resolve_one_dep(
	resolver: &Resolver,
	name: &SmolStr,
	range: &str,
	dep_type: DepType,
) -> Result<DepNode> {
	let range = resolver.overrides.get(name).map(|s| s.as_str()).unwrap_or(range);
	let source = oxpm_common::SourceType::parse(range)
		.map_err(|e| Error::Source(oxpm_common::SourceError(e.0.to_string())))?;

	let dep_scope = extract_scope(name);

	match source {
		oxpm_common::SourceType::Registry(r) => {
			let registry = resolver.get_registry_for_scope(dep_scope.as_ref())?;
			crate::source::resolve_registry(name, range, r, dep_type, &registry, &resolver.cache)
				.await
		}
		oxpm_common::SourceType::File(f) => {
			crate::source::resolve_file(name, range, f, dep_type, &resolver.cache).await
		}
		oxpm_common::SourceType::Link(l) => {
			crate::source::resolve_link(name, range, l, dep_type, &resolver.cache).await
		}
		oxpm_common::SourceType::Tarball(t) => {
			crate::source::resolve_tarball(name, range, t, dep_type, &resolver.cache).await
		}
		_ => Err(Error::Source(oxpm_common::SourceError(source.to_source_string().to_string()))),
	}
}

fn extract_scope(name: &SmolStr) -> Option<ScopeType> {
	let s = name.as_str();
	if s.starts_with('@')
		&& let Some(at_pos) = s.find('/')
	{
		let scope_str = &s[1..at_pos];
		return Some(ScopeType::from(scope_str));
	}
	None
}
