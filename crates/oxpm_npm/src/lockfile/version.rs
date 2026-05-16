use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockfileVersion {
	V3,
}

impl LockfileVersion {
	pub const fn as_u32(self) -> u32 {
		3
	}
}

impl TryFrom<u64> for LockfileVersion {
	type Error = Error;

	fn try_from(value: u64) -> Result<Self, Self::Error> {
		match value {
			3 => Ok(Self::V3),
			other => Err(Error::UnsupportedLockfileVersion(other)),
		}
	}
}

impl Serialize for LockfileVersion {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_u32(self.as_u32())
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
