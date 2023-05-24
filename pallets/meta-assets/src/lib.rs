#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::BoundedVec;
pub use pallet::*;
use sp_core::Get;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;
	use serde_json::Value;
	// use valico::json_schema; === CANT USE BECAUSE VALICO IS STD ONLY

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub type AssetId = u32;

	pub type BoundedString<T> = BoundedVec<u8, <T as Config>::StringLimit>;

	pub type CollectionId<T> = BoundedString<T>;

	pub type BoundedJson<T> = BoundedVec<u8, <T as Config>::JsonLimit>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Pallet configuration trait
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type StringLimit: Get<u32>;

		type JsonLimit: Get<u32>;
	}

	// Collections
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Collection<T: Config> {
		pub name: BoundedString<T>,
		pub description: BoundedString<T>,
		pub author: <T as frame_system::Config>::AccountId,
		pub schema: BoundedJson<T>,
		pub items_count: u32,
	}

	#[pallet::storage]
	#[pallet::getter(fn collections)]
	pub type CollectionsStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::Hash,       // Collection ID
		Collection<T>, // Collection
	>;

	// Assets
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct AssetItem<T: Config> {
		pub name: BoundedString<T>,
		pub owner: <T as frame_system::Config>::AccountId,
		pub meta: BoundedJson<T>,
	}

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub type AssetsStore<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::Hash, // Collection ID
		Twox64Concat,
		T::Hash,      //Asset hash
		AssetItem<T>, // Asset
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetWasStored { asset_hash: T::Hash, owner: T::AccountId },
		AssetWasTransferred { asset_hash: T::Hash, from: T::AccountId, to: T::AccountId },
		AssetWasRemoved { asset_hash: T::Hash, owner: T::AccountId },
		MetaUpdated { asset_hash: T::Hash, owner: T::AccountId },
		CollectionCreated { collection_hash: T::Hash, owner: T::AccountId },
		CollectionUpdated { collection_hash: T::Hash, owner: T::AccountId },
		CollectionRemoved { collection_hash: T::Hash, owner: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		Unauthorized,
		InvalidHash,
		ShortNameProvided,
		LongNameProvided,
		InvalidCollection,
		CollectionAlreadyExists,
		AssetAlreadyExists,
		InvalidJson,
		InvalidJsonByCollectionSchema,
		SomeAssetsExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Method to create new collection
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_collection(
			origin: OriginFor<T>,
			name: BoundedString<T>,
			description: BoundedString<T>,
			schema: BoundedJson<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let json = serde_json::from_str::<Value>(
				sp_core::sp_std::str::from_utf8(schema.clone().as_slice()).unwrap(),
			);
			ensure!(json.is_ok(), Error::<T>::InvalidJson);
			ensure!(name.len() > 3, Error::<T>::ShortNameProvided);
			ensure!(name.len() < 200, Error::<T>::LongNameProvided);

			let collection = Collection {
				name: name.clone(),
				description: description.clone(),
				author: owner.clone(),
				schema: schema.clone(),
				items_count: 0,
			};

			// Check if hash is already used
			let collection_hash = T::Hashing::hash_of(&collection);
			ensure!(
				!<CollectionsStore<T>>::contains_key(collection_hash),
				Error::<T>::CollectionAlreadyExists
			);

			// Update the storage.
			<CollectionsStore<T>>::insert(collection_hash, collection);

			// Emit an event.
			Self::deposit_event(Event::CollectionCreated { collection_hash, owner });

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// Method to add new asset to collection
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(2).ref_time())]
		pub fn add_asset(
			origin: OriginFor<T>,
			asset_name: BoundedString<T>,
			collection_hash: T::Hash,
			meta: BoundedJson<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			ensure!(asset_name.len() > 3, Error::<T>::ShortNameProvided);
			ensure!(asset_name.len() < 200, Error::<T>::LongNameProvided);

			// Get collection
			let mut collection =
				CollectionsStore::<T>::get(collection_hash).ok_or(Error::<T>::InvalidCollection)?;

			// Meta json validation
			let json = serde_json::from_str::<Value>(
				sp_core::sp_std::str::from_utf8(meta.clone().as_slice()).unwrap(),
			);
			ensure!(json.is_ok(), Error::<T>::InvalidJson);

			// ======= CANT DO BECAUSE VALICO IS ONLY STD FOR NOW
			// Gather collection JSON schema
			// let schema = collection.schema;
			// let j_schema = serde_json::from_str::<Value>(
			// 	sp_core::sp_std::str::from_utf8(schema.clone().as_slice()).unwrap(),
			// );
			// let mut scope = json_schema::Scope::new();
			// let r_schema = scope.compile_and_return(j_schema.unwrap(), true).ok().unwrap();

			// // Validate JSON against schema
			// ensure!(
			// 	r_schema
			// 		.validate(
			// 			&serde_json::from_slice::<Value>(meta.clone().to_vec().as_slice()).unwrap(),
			// 		)
			// 		.is_valid(),
			// 	Error::<T>::InvalidJsonByCollectionSchema
			// );

			// Create asset
			let asset = AssetItem { name: asset_name.clone(), owner, meta: meta.clone() };
			let asset_hash = T::Hashing::hash_of(&asset);
			ensure!(
				!<AssetsStore<T>>::contains_key(collection_hash, asset_hash),
				Error::<T>::AssetAlreadyExists,
			);
			// Update storage.
			<AssetsStore<T>>::insert(collection_hash, asset_hash, asset.clone());

			// Update collection
			collection.items_count += 1;
			<CollectionsStore<T>>::insert(collection_hash, collection);

			// Emit the events
			Self::deposit_event(Event::CollectionUpdated {
				collection_hash,
				owner: asset.owner.clone(),
			});

			Self::deposit_event(Event::AssetWasStored { asset_hash, owner: asset.owner });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// Method to update asset metadata
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn update_meta(
			origin: OriginFor<T>,
			collection_hash: T::Hash,
			asset_hash: T::Hash,
			meta: BoundedJson<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			// Get collection
			let collection =
				CollectionsStore::<T>::get(collection_hash).ok_or(Error::<T>::InvalidCollection)?;

			// Check if collection owner is trying to change metadata
			ensure!(collection.author == owner, Error::<T>::Unauthorized);

			// Meta json validation
			let json = serde_json::from_str::<Value>(
				sp_core::sp_std::str::from_utf8(meta.clone().as_slice()).unwrap(),
			);
			ensure!(json.is_ok(), Error::<T>::InvalidJson);

			// ======= CANT DO BECAUSE VALICO IS ONLY STD FOR NOW
			// Gather collection JSON schema
			// let schema = collection.unwrap().schema;
			// let j_schema = serde_json::from_str::<Value>(
			// 	sp_core::sp_std::str::from_utf8(schema.clone().as_slice()).unwrap(),
			// );
			// let mut scope = json_schema::Scope::new();
			// let r_schema = scope.compile_and_return(j_schema.unwrap(), true).ok().unwrap();
			//
			// // Validate JSON against schema
			// ensure!(
			// 	r_schema
			// 		.validate(
			// 			&serde_json::from_slice::<Value>(meta.clone().to_vec().as_slice()).unwrap(),
			// 		)
			// 		.is_valid(),
			// 	Error::<T>::InvalidJsonByCollectionSchema
			// );

			ensure!(
				<AssetsStore<T>>::contains_key(collection_hash, asset_hash),
				Error::<T>::InvalidHash
			);

			let asset = <AssetsStore<T>>::get(collection_hash, asset_hash).unwrap();

			let new_asset = AssetItem { name: asset.name, owner: asset.owner, meta: meta.clone() };

			<AssetsStore<T>>::insert(collection_hash, asset_hash, new_asset);

			// Emit an event.
			Self::deposit_event(Event::MetaUpdated { asset_hash, owner });

			Ok(())
		}

		// Method to transfer asset to another account
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer_asset(
			origin: OriginFor<T>,
			collection_hash: T::Hash,
			asset_hash: T::Hash,
			destination: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let asset = <AssetsStore<T>>::get(collection_hash, asset_hash)
				.ok_or(Error::<T>::InvalidHash)?;

			ensure!(asset.owner == owner, Error::<T>::Unauthorized);

			let new_asset =
				AssetItem { name: asset.name, owner: destination.clone(), meta: asset.meta };

			<AssetsStore<T>>::insert(collection_hash, asset_hash, new_asset);

			// Emit an event.
			Self::deposit_event(Event::AssetWasTransferred {
				asset_hash,
				from: owner,
				to: destination,
			});

			Ok(())
		}

		// Method to remove an asset from the store.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(2).ref_time())]
		pub fn remove_asset(
			origin: OriginFor<T>,
			collection_hash: T::Hash,
			asset_hash: T::Hash,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let mut collection =
				<CollectionsStore<T>>::get(collection_hash).ok_or(Error::<T>::InvalidCollection)?;

			let asset = <AssetsStore<T>>::get(collection_hash, asset_hash)
				.ok_or(Error::<T>::InvalidHash)?;

			ensure!(asset.owner == owner, Error::<T>::Unauthorized);

			<AssetsStore<T>>::remove(collection_hash, asset_hash);

			// Update collection
			collection.items_count += 1;
			<CollectionsStore<T>>::insert(collection_hash, collection);
			Self::deposit_event(Event::CollectionUpdated { collection_hash, owner: asset.owner });

			// Emit an event.
			Self::deposit_event(Event::AssetWasRemoved { asset_hash, owner });

			Ok(())
		}

		// Method to remove collection from the store.
		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn remove_collection(origin: OriginFor<T>, collection_hash: T::Hash) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let collection =
				<CollectionsStore<T>>::get(collection_hash).ok_or(Error::<T>::InvalidHash)?;

			ensure!(collection.author == owner, Error::<T>::Unauthorized);

			// Check if collection is empty
			ensure!(collection.items_count == 0, Error::<T>::SomeAssetsExists);

			<CollectionsStore<T>>::remove(collection_hash);

			// Emit an event.
			Self::deposit_event(Event::CollectionRemoved { collection_hash, owner });

			Ok(())
		}
	}
}
