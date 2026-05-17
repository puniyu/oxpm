use serde::{Deserialize, Deserializer};

use crate::LockFileError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockfileVersion {
	V3,
}

impl LockfileVersion {
	pub const fn as_u32(self) -> u32 {
		match self {
			Self::V3 => 3,
		}
	}
}

impl TryFrom<u64> for LockfileVersion {
	type Error = LockFileError;

	fn try_from(value: u64) -> Result<Self, Self::Error> {
		match value {
			3 => Ok(Self::V3),
			other => Err(LockFileError::UnsupportedLockfileVersion(other)),
		}
	}
}


impl<'de> Deserialize<'de> for LockfileVersion {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let v = u64::deserialize(deserializer)?;
		Self::try_from(v)
			.map_err(|_| serde::de::Error::custom(format!("unsupported lockfileVersion: {v}")))
	}
}
