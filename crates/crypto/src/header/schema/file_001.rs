use bincode::impl_borrow_decode;

use crate::{
	crypto::{Decryptor, Encryptor},
	header::file::{Header, HeaderObjectType},
	primitives::{generate_bytes, FILE_KEY_CONTEXT},
	types::{Aad, Algorithm, EncryptedKey, HashingAlgorithm, Key, Nonce, Params, Salt},
	Error, Protected, Result,
};

const KEYSLOT_LIMIT: usize = 2;
const OBJECT_LIMIT: usize = 2;

#[derive(Clone, bincode::Encode, bincode::Decode)]
pub struct FileHeader001 {
	pub aad: Aad,
	pub algorithm: Algorithm,
	pub nonce: Nonce,
	pub keyslots: KeyslotArea001,
	pub objects: Vec<FileHeaderObject001>,
}

/// A keyslot - 96 bytes (as of V1), and contains all the information for future-proofing while keeping the size reasonable
#[derive(bincode::Encode, bincode::Decode, Clone)]
pub struct Keyslot001 {
	pub enabled: bool,
	pub hashing_algorithm: HashingAlgorithm, // password hashing algorithm
	pub salt: Salt, // the salt used for deriving a KEK from a (key/content salt) hash
	pub content_salt: Salt,
	pub master_key: EncryptedKey, // encrypted
	pub nonce: Nonce,
}

impl Keyslot001 {
	pub fn disabled() -> Self {
		Self {
			enabled: false,
			content_salt: Salt::generate(),
			hashing_algorithm: HashingAlgorithm::Argon2id(Params::Standard),
			master_key: EncryptedKey(generate_bytes()),
			salt: Salt::generate(),
			nonce: Nonce::generate_xchacha(),
		}
	}
}

#[derive(Clone)]
pub struct KeyslotArea001(Vec<Keyslot001>);

impl TryFrom<Vec<Keyslot001>> for KeyslotArea001 {
	type Error = Error;

	fn try_from(value: Vec<Keyslot001>) -> std::result::Result<Self, Self::Error> {
		if value.len() > KEYSLOT_LIMIT {
			return Err(Error::TooManyKeyslots);
		}

		Ok(Self(value))
	}
}

impl bincode::Decode for KeyslotArea001 {
	fn decode<D: bincode::de::Decoder>(
		decoder: &mut D,
	) -> std::result::Result<Self, bincode::error::DecodeError> {
		let keyslots: Vec<Keyslot001> = (0..KEYSLOT_LIMIT)
			.filter_map(|_| {
				bincode::decode_from_reader(decoder.reader(), bincode::config::standard())
					.ok()
					.filter(|x: &Keyslot001| x.enabled)
			})
			.collect();

		Self::try_from(keyslots).map_err(Into::into)
	}
}

impl_borrow_decode!(KeyslotArea001);

impl bincode::Encode for KeyslotArea001 {
	fn encode<E: bincode::enc::Encoder>(
		&self,
		encoder: &mut E,
	) -> std::result::Result<(), bincode::error::EncodeError> {
		if self.0.len() > KEYSLOT_LIMIT {
			return Err(Error::TooManyKeyslots.into());
		}

		self.0.iter().try_for_each(|k| k.encode(encoder))?;

		(0..KEYSLOT_LIMIT - self.0.len())
			.try_for_each(|_| Keyslot001::disabled().encode(encoder))?;

		Ok(())
	}
}

#[derive(Clone, bincode::Encode, bincode::Decode)]
pub struct FileHeaderObject001 {
	pub object_type: HeaderObjectType,
	pub nonce: Nonce,
	pub data: Vec<u8>,
}

impl Keyslot001 {
	#[allow(clippy::needless_pass_by_value)]
	pub async fn new(
		algorithm: Algorithm,
		hashing_algorithm: HashingAlgorithm,
		content_salt: Salt,
		hashed_key: Key,
		master_key: Key,
	) -> Result<Self> {
		let nonce = Nonce::generate(algorithm)?;

		let salt = Salt::generate();

		let encrypted_master_key = EncryptedKey::try_from(
			Encryptor::encrypt_bytes(
				Key::derive(hashed_key, salt, FILE_KEY_CONTEXT),
				nonce,
				algorithm,
				master_key.expose(),
				&[],
			)
			.await?,
		)?;

		Ok(Self {
			enabled: true,
			hashing_algorithm,
			salt,
			content_salt,
			master_key: encrypted_master_key,
			nonce,
		})
	}

	#[allow(clippy::needless_pass_by_value)]
	async fn decrypt(&self, algorithm: Algorithm, key: Key) -> Result<Key> {
		Key::try_from(
			Decryptor::decrypt_bytes(
				Key::derive(key, self.salt, FILE_KEY_CONTEXT),
				self.nonce,
				algorithm,
				&self.master_key,
				&[],
			)
			.await?,
		)
	}
}

impl FileHeader001 {
	// TODO(brxken128): make the AAD not static
	// should be brought in from the raw file bytes but bincode makes that harder
	// as the first 32~ bytes of the file *may* change
	pub fn new(algorithm: Algorithm) -> Result<Self> {
		let f = Self {
			aad: Aad::generate(),
			algorithm,
			nonce: Nonce::generate(algorithm)?,
			keyslots: KeyslotArea001(vec![]),
			objects: vec![],
		};

		Ok(f)
	}
}

impl FileHeaderObject001 {
	pub async fn new(
		object_type: HeaderObjectType,
		algorithm: Algorithm,
		master_key: Key,
		aad: Aad,
		data: &[u8],
	) -> Result<Self> {
		let nonce = Nonce::generate(algorithm)?;

		let encrypted_data =
			Encryptor::encrypt_bytes(master_key, nonce, algorithm, data, &aad).await?;

		let object = Self {
			object_type,
			nonce,
			data: encrypted_data,
		};

		Ok(object)
	}

	async fn decrypt(
		&self,
		algorithm: Algorithm,
		aad: Aad,
		master_key: Key,
	) -> Result<Protected<Vec<u8>>> {
		let pvm =
			Decryptor::decrypt_bytes(master_key, self.nonce, algorithm, &self.data, &aad).await?;

		Ok(pvm)
	}
}

#[async_trait::async_trait]
impl Header for FileHeader001 {
	fn serialize(&self) -> Result<Vec<u8>> {
		bincode::encode_to_vec(self, bincode::config::standard()).map_err(Error::BincodeEncode)
	}

	async fn decrypt_object(&self, index: usize, master_key: Key) -> Result<Protected<Vec<u8>>> {
		if index >= self.objects.len() || self.objects.is_empty() {
			return Err(Error::Index);
		}

		self.objects[index]
			.decrypt(self.algorithm, self.aad, master_key)
			.await
	}

	async fn add_keyslot(
		&mut self,
		hashing_algorithm: HashingAlgorithm,
		content_salt: Salt,
		hashed_key: Key,
		master_key: Key,
	) -> Result<()> {
		if self.keyslots.0.len() + 1 > KEYSLOT_LIMIT {
			return Err(Error::TooManyKeyslots);
		}

		self.keyslots.0.push(
			Keyslot001::new(
				self.algorithm,
				hashing_algorithm,
				content_salt,
				hashed_key,
				master_key,
			)
			.await?,
		);

		Ok(())
	}

	async fn add_object(
		&mut self,
		object_type: HeaderObjectType,
		master_key: Key,
		data: &[u8],
	) -> Result<()> {
		if self.objects.len() + 1 > OBJECT_LIMIT {
			return Err(Error::TooManyObjects);
		}

		self.objects.push(
			FileHeaderObject001::new(object_type, self.algorithm, master_key, self.aad, data)
				.await?,
		);
		Ok(())
	}

	#[allow(clippy::needless_pass_by_value)]
	async fn decrypt_master_key(&self, keys: Vec<Key>) -> Result<Key> {
		if self.keyslots.0.is_empty() {
			return Err(Error::NoKeyslots);
		}

		for hashed_key in keys {
			for v in &self.keyslots.0 {
				if let Ok(key) = v.decrypt(self.algorithm, hashed_key.clone()).await {
					return Ok(key);
				}
			}
		}

		Err(Error::IncorrectPassword)
	}

	#[allow(clippy::needless_pass_by_value)]
	async fn decrypt_master_key_with_password(&self, password: Protected<Vec<u8>>) -> Result<Key> {
		if self.keyslots.0.is_empty() {
			return Err(Error::NoKeyslots);
		}

		for v in &self.keyslots.0 {
			let key = v
				.hashing_algorithm
				.hash(password.clone(), v.content_salt, None)
				.map_err(|_| Error::PasswordHash)?;

			if let Ok(key) = v.decrypt(self.algorithm, key).await {
				return Ok(key);
			}
		}

		Err(Error::IncorrectPassword)
	}

	fn get_aad(&self) -> Aad {
		self.aad
	}

	fn get_nonce(&self) -> Nonce {
		self.nonce
	}

	fn get_algorithm(&self) -> Algorithm {
		self.algorithm
	}

	fn count_objects(&self) -> usize {
		self.objects.len()
	}

	fn count_keyslots(&self) -> usize {
		self.keyslots.0.len()
	}
}
