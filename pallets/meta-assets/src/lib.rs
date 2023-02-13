#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{inherent::Vec, pallet_prelude::*, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AssetItem<T: Config> {
		pub name: Vec<u8>,
		pub owner: <T as frame_system::Config>::AccountId,
	}

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub type AssetsStore<T: Config> = StorageMap<_, Twox64Concat, T::Hash, AssetItem<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetWasStored(Vec<u8>, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		ShortNameProvided,
		LongNameProvided,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_asset(origin: OriginFor<T>, asset_name: Vec<u8>) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			ensure!(asset_name.len() > 3, Error::<T>::ShortNameProvided);
			ensure!(asset_name.len() < 32, Error::<T>::LongNameProvided);

			let asset = AssetItem { name: asset_name.clone(), owner: owner.clone() };

			let asset_hash = T::Hashing::hash_of(&asset);

			// Update storage.
			<AssetsStore<T>>::insert(asset_hash, asset);

			// Emit an event.
			Self::deposit_event(Event::AssetWasStored(asset_name, owner));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer_asset(
			origin: OriginFor<T>,
			hash: T::Hash,
			destination: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let asset = <AssetsStore<T>>::get(hash).ok_or(Error::<T>::NoneValue)?;

			ensure!(asset.owner == owner, Error::<T>::NoneValue);

			let new_asset = AssetItem { name: asset.name, owner: destination.clone() };

			<AssetsStore<T>>::insert(hash, new_asset);

			Ok(())
		}
	}
}
