use smol_str::SmolStr;

use oxpm_registry::{Package, PackageVersion};
use oxpm_semver::Version;

use crate::error::Error;
use crate::Result;


pub fn select_version(
    name: &SmolStr,
    range: &str,
    pkg_info: &Package,
) -> Result<(Version, PackageVersion)> {
    use oxpm_semver::VersionRangeKind;

    let range_kind = VersionRangeKind::parse(range)?;

    if matches!(range_kind, VersionRangeKind::Any)
        && let Some(latest) = pkg_info.dist_tags.get("latest")
    {
        let latest_version = Version::parse(latest.as_str())?;
        let pv = pkg_info
            .versions
            .get(latest.as_str())
            .ok_or_else(|| Error::VersionNotFound {
                name: name.clone(),
                range: latest.clone(),
            })?;
        return Ok((latest_version, pv.clone()));
    }

    let mut best_version: Option<Version> = None;
    let mut best_pv: Option<PackageVersion> = None;

    for (ver_str, pv) in &pkg_info.versions {
        let v = Version::parse(ver_str.as_str())?;
        if range_kind.matches(&v) {
            match &best_version {
                None => {
                    best_version = Some(v);
                    best_pv = Some(pv.clone());
                }
                Some(best) if v > *best => {
                    best_version = Some(v);
                    best_pv = Some(pv.clone());
                }
                _ => {}
            }
        }
    }

    best_version
        .zip(best_pv)
        .ok_or_else(|| Error::VersionNotFound {
            name: name.clone(),
            range: SmolStr::new(range),
        })
}