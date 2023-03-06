#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::BoundedVec;
pub use pallet::*;
use sp_core::Get;

#[cfg(test)]
mod mock;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub type AssetId = u32;

	pub type BoundedString<T> = BoundedVec<u8, <T as Config>::StringLimit>;

	pub type CollectionId<T> = BoundedString<T>;

	pub type BoundedJson<T> = BoundedVec<u8, <T as Config>::JsonLimit>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type StringLimit: Get<u32>;

		type JsonLimit: Get<u32>;
	}

	/// Collections

	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Collection<T: Config> {
		pub description: BoundedString<T>,
		pub author: <T as frame_system::Config>::AccountId,
		pub schema: BoundedJson<T>,
	}

	#[pallet::storage]
	#[pallet::getter(fn collections)]
	pub type CollectionsStore<T: Config> =
		StorageMap<_, Twox64Concat, BoundedString<T>, Collection<T>>;

	/// Assets
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct AssetItem<T: Config> {
		pub name: BoundedString<T>,
		pub owner: <T as frame_system::Config>::AccountId,
	}

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub type AssetsStore<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		BoundedString<T>, // Collection name
		Twox64Concat,
		T::Hash, //Asset hash
		AssetItem<T>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn assets_meta)]
	pub type MetadataStore<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::Hash,
		Twox64Concat,
		T::AccountId,
		Option<BoundedJson<T>>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetWasStored(T::Hash, T::AccountId), // (asset_hash, owner)
		AssetWasTransferred(T::Hash, T::AccountId, T::AccountId), // (asset_hash, from, to)
		AssetWasRemoved(T::Hash, T::AccountId), // (asset_hash, owner)
		MetaUpdated(T::Hash, T::AccountId),    // (asset_hash, owner)
		AdminRegistered(T::Hash, T::AccountId, T::AccountId), // (asset_hash, owner, new_admin)
		AdminRemoved(T::Hash, T::AccountId, T::AccountId), // (asset_hash, owner, removed_admin)
		CollectionCreated(BoundedString<T>, T::AccountId), // (collection_name, creator)
		NewAssetInCollection(BoundedString<T>, T::Hash), // (collection_name, asset_hash)
	}

	#[pallet::error]
	pub enum Error<T> {
		Unauthorized,
		InvalidHash,
		InvalidAddress,
		ShortNameProvided,
		LongNameProvided,
		AlreadyRegistered,
		InvalidCollection,
		CollectionNameExists,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_asset(
			origin: OriginFor<T>,
			asset_name: BoundedString<T>,
			collection: BoundedString<T>,
			meta: Option<BoundedJson<T>>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			ensure!(asset_name.len() > 3, Error::<T>::ShortNameProvided);
			ensure!(asset_name.len() < 32, Error::<T>::LongNameProvided);

			// Check if collection exists
			ensure!(
				CollectionsStore::<T>::contains_key(collection.clone()),
				Error::<T>::InvalidCollection
			);

			let asset = AssetItem { name: asset_name.clone(), owner: owner.clone() };

			let asset_hash = T::Hashing::hash_of(&asset);

			// Update storage.
			<AssetsStore<T>>::insert(collection.clone(), asset_hash, asset.clone());
			<MetadataStore<T>>::insert(asset_hash, owner.clone(), meta.clone());

			// Emit an event.
			Self::deposit_event(Event::AssetWasStored(asset_hash, asset.owner.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// Method to remove an asset from the store.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn remove_asset(
			origin: OriginFor<T>,
			collection: BoundedString<T>,
			hash: T::Hash,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let asset =
				<AssetsStore<T>>::get(collection.clone(), hash).ok_or(Error::<T>::InvalidHash)?;

			ensure!(asset.owner == owner, Error::<T>::Unauthorized);

			<AssetsStore<T>>::remove(collection.clone(), hash);
			<MetadataStore<T>>::remove(hash, owner.clone());

			// Emit an event.
			Self::deposit_event(Event::AssetWasRemoved(hash, owner.clone()));

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer_asset(
			origin: OriginFor<T>,
			collection: BoundedString<T>,
			hash: T::Hash,
			destination: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let asset =
				<AssetsStore<T>>::get(collection.clone(), hash).ok_or(Error::<T>::InvalidHash)?;

			ensure!(asset.owner == owner, Error::<T>::Unauthorized);

			let new_asset = AssetItem { name: asset.name, owner: destination.clone() };

			<AssetsStore<T>>::insert(collection.clone(), hash, new_asset);

			// Emit an event.
			Self::deposit_event(Event::AssetWasTransferred(
				hash,
				owner.clone(),
				destination.clone(),
			));

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn update_meta(
			origin: OriginFor<T>,
			collection: BoundedString<T>,
			hash: T::Hash,
			meta: Option<BoundedJson<T>>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			ensure!(<AssetsStore<T>>::contains_key(collection, hash), Error::<T>::InvalidHash);

			// Check if admin is registered
			ensure!(
				<MetadataStore<T>>::contains_key(hash, owner.clone()),
				Error::<T>::Unauthorized
			);

			<MetadataStore<T>>::insert(hash, owner.clone(), meta.clone());

			// Emit an event.
			Self::deposit_event(Event::MetaUpdated(hash, owner.clone()));

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_admin(
			origin: OriginFor<T>,
			collection: BoundedString<T>,
			hash: T::Hash,
			admin_address: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let asset = <AssetsStore<T>>::get(collection, hash).ok_or(Error::<T>::InvalidHash)?;

			ensure!(asset.owner == owner, Error::<T>::Unauthorized);

			// Check if admin is already registered
			ensure!(
				!<MetadataStore<T>>::contains_key(hash, admin_address.clone()),
				Error::<T>::AlreadyRegistered
			);

			<MetadataStore<T>>::insert(hash, admin_address.clone(), None::<BoundedJson<T>>);

			// Emit an event.
			Self::deposit_event(Event::AdminRegistered(hash, owner.clone(), admin_address.clone()));

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn remove_admin(
			origin: OriginFor<T>,
			collection: BoundedString<T>,
			hash: T::Hash,
			admin_address: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let asset = <AssetsStore<T>>::get(collection, hash).ok_or(Error::<T>::InvalidHash)?;

			ensure!(asset.owner == owner, Error::<T>::Unauthorized);

			// Check if admin is already registered
			ensure!(
				<MetadataStore<T>>::contains_key(hash, admin_address.clone()),
				Error::<T>::InvalidAddress
			);

			<MetadataStore<T>>::remove(hash, admin_address.clone());

			// Emit an event.
			Self::deposit_event(Event::AdminRemoved(hash, owner.clone(), admin_address.clone()));

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_collection(
			origin: OriginFor<T>,
			name: BoundedString<T>,
			description: BoundedString<T>,
			schema: BoundedJson<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let json: Value = serde_json::from_str(schema);

			ensure!(name.len() > 3, Error::<T>::ShortNameProvided);
			ensure!(name.len() < 32, Error::<T>::LongNameProvided);

			// Check if name is unique
			ensure!(
				!<CollectionsStore<T>>::contains_key(name.clone()),
				Error::<T>::CollectionNameExists
			);

			let collection = Collection {
				description: description.clone(),
				author: owner.clone(),
				schema: schema.clone(),
			};

			// Update storage.
			<CollectionsStore<T>>::insert(name.clone(), collection.clone());

			// Emit an event.
			Self::deposit_event(Event::CollectionCreated(name.clone(), owner.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}
