use std::path::Path;

use ecow::EcoVec;
use oxpm_semver::Version;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol_str::SmolStr;

use url::Url;

mod types;
pub use types::*;
mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// npm `package.json` 配置文件的完整表示。
///
/// 参考: <https://docs.npmjs.com/cli/v11/configuring-npm/package-json>
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
	/// 包名，必须小写且不超过 214 个字符。作用域包以 `@scope/` 为前缀。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<SmolStr>,
	/// 语义化版本号，必须可被 `node-semver` 解析。与 `name` 一起构成唯一标识。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<Version>,
	/// 包的简短描述，用于 `npm search` 中帮助用户发现包。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<SmolStr>,
	/// 关键词数组，用于 `npm search` 中的索引和分类。
	#[serde(default, skip_serializing_if = "EcoVec::is_empty")]
	pub keywords: EcoVec<SmolStr>,
	/// 项目主页的 URL。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub homepage: Option<Url>,
	/// 问题追踪器的地址和/或报告问题的邮箱。可以是 URL 字符串或 `{url, email}` 对象。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bugs: Option<PackageBugs>,
	/// SPDX 许可证表达式，例如 `"MIT"` 或 `"(MIT OR Apache-2.0)"`。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<SmolStr>,
	/// 包的作者。可以是 `"Name <email> (url)"` 格式的字符串或 `{name, email, url}` 对象。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub author: Option<Person>,
	/// 贡献者列表，每个元素的格式与 `author` 相同。
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub contributors: Vec<Person>,
	/// 维护者列表，每个元素的格式与 `author` 相同。
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub maintainers: Vec<Person>,
	/// 资助信息，可以是单个 URL/对象，也可以是数组。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub funding: Option<PackageFundings>,
	/// 包发布时包含的文件 glob 模式数组。省略时默认包含所有文件。
	#[serde(default, skip_serializing_if = "EcoVec::is_empty")]
	pub files: EcoVec<SmolStr>,
	/// CommonJS 模块的主入口文件路径，相对于包根目录。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub main: Option<SmolStr>,
	/// 浏览器环境下的入口文件，用于替代 `main` 中指定的 Node.js 模块。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub browser: Option<SmolStr>,
	/// 可执行文件映射。字符串时以 `name` 为命令名；对象时为 `{命令名: 文件路径}` 映射。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bin: Option<PackageBin>,
	/// man 手册页文件路径，字符串或字符串数组。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub man: Option<PackageMan>,
	/// 目录结构提示，指定 `bin` 和 `man` 目录的位置。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub directories: Option<PackageDirectories>,
	/// 源码仓库地址。可以是 `"github:user/repo"` 简写或 `{type, url, directory}` 对象。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub repository: Option<PackageRepository>,
	/// 生命周期脚本命令映射，通过 `npm run <script>` 执行。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scripts: Option<IndexMap<SmolStr, SmolStr>>,
	/// 用于设置脚本中可引用的配置参数，通过 `npm_package_config_<key>` 环境变量访问。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub config: Option<IndexMap<SmolStr, Value>>,
	/// 生产环境依赖，键为包名，值为 semver 范围。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dependencies: Option<IndexMap<SmolStr, SmolStr>>,
	/// 开发环境依赖，仅在开发和测试时安装。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dev_dependencies: Option<IndexMap<SmolStr, SmolStr>>,
	/// 对等依赖，声明本包兼容的宿主包版本范围，由消费者提供。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub peer_dependencies: Option<IndexMap<SmolStr, SmolStr>>,
	/// 对等依赖的元数据，例如 `{"optional": true}` 表示该对等依赖是可选的。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub peer_dependencies_meta: Option<IndexMap<SmolStr, IndexMap<SmolStr, Value>>>,
	/// 可选依赖，安装失败不会导致整个安装过程失败。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub optional_dependencies: Option<IndexMap<SmolStr, SmolStr>>,
	/// 捆绑依赖列表，发布时将这些包打包进 tarball 中。
	#[serde(default, skip_serializing_if = "EcoVec::is_empty")]
	pub bundle_dependencies: EcoVec<SmolStr>,
	/// 依赖版本覆盖，用于替换依赖树中特定包的版本，支持嵌套。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub overrides: Option<IndexMap<SmolStr, OverrideValue>>,
	/// 引擎兼容性约束，例如 `{"node": ">=18"}`。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub engines: Option<IndexMap<SmolStr, SmolStr>>,
	/// 包可以运行的操作系统列表，前缀 `!` 表示排除。
	#[serde(default, skip_serializing_if = "EcoVec::is_empty")]
	pub os: EcoVec<SmolStr>,
	/// 包可以运行的 CPU 架构列表，前缀 `!` 表示排除。
	#[serde(default, skip_serializing_if = "EcoVec::is_empty")]
	pub cpu: EcoVec<SmolStr>,
	/// 设为 `true` 可防止包被意外发布到 npm 仓库。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub private: Option<bool>,
	/// 发布时使用的配置，例如 `{"registry": "https://npm.pkg.github.com"}`。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub publish_config: Option<IndexMap<SmolStr, Value>>,
	/// 工作区 glob 模式数组，定义 monorepo 中子包的位置。
	#[serde(default, skip_serializing_if = "EcoVec::is_empty")]
	pub workspaces: EcoVec<SmolStr>,
	/// Node.js 条件导出映射，定义包的公共 API 入口点和条件解析规则。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub exports: Option<PackageExports>,
	/// 包内部导入映射，以 `#` 开头的子路径别名，仅包自身可用。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub imports: Option<IndexMap<SmolStr, PackageExportsEntry>>,
	/// 模块系统类型：`"commonjs"`（默认）或 `"module"`（ESM）。
	#[serde(rename = "type", skip_serializing_if = "Option::is_none")]
	pub module_type: Option<PackageType>,
}

impl PackageJson {
	pub fn load_from_path(path: impl AsRef<Path>) -> Result<PackageJson> {
		let content = std::fs::read_to_string(path)?;
		let pkg = serde_json::from_str(&content)?;
		Ok(pkg)
	}

	pub fn load_from_str(content: &str) -> Result<PackageJson> {
		let pkg = serde_json::from_str(content)?;
		Ok(pkg)
	}
}