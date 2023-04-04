use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, pallet_prelude::DispatchResult};
use sp_core::H256;

fn create_collection(name: &[u8], description: &[u8], schema: &[u8]) -> DispatchResult {
	let collection_name: BoundedString<Test> = name.to_vec().try_into().unwrap();

	let description: BoundedString<Test> = description.to_vec().try_into().unwrap();

	let schema: BoundedJson<Test> = schema.to_vec().try_into().unwrap();

	MetaAssetsModule::create_collection(
		RuntimeOrigin::signed(ALICE),
		collection_name.clone(),
		description.clone(),
		schema.clone(),
	)
}

fn create_asset(collection_hash: H256, name: &[u8], meta: &[u8]) -> DispatchResult {
	let asset_name: BoundedString<Test> = name.to_vec().try_into().unwrap();

	let asset_meta: BoundedJson<Test> = meta.to_vec().try_into().unwrap();

	MetaAssetsModule::add_asset(
		RuntimeOrigin::signed(ALICE),
		asset_name.clone(),
		collection_hash.clone(),
		asset_meta.clone(),
	)
}
#[test]
fn it_should_create_collection() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));
		assert_eq!(get_pallet_events().len(), 1);
	})
}

#[test]
fn it_should_not_create_duplicate_collection() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));
		assert_noop!(
			create_collection(b"Collection 1", b"Description xyz", b"{}"),
			Error::<Test>::CollectionAlreadyExists
		);
	})
}
#[test]
fn it_should_not_create_collection_with_invalid_schema() {
	ExtBuilder.build().execute_with(|| {
		assert_noop!(
			create_collection(b"Collection 1", b"Description xyz", b"{lol"),
			Error::<Test>::InvalidJson
		);
	})
}

#[test]
fn it_should_remove_collection() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));

		let collection_hash = match get_latest_event() {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};
		assert_ok!(MetaAssetsModule::remove_collection(
			RuntimeOrigin::signed(ALICE),
			collection_hash
		));

		assert_eq!(get_pallet_events().len(), 2);
	})
}

#[test]
fn it_should_not_remove_collection() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));

		let collection_hash = match get_latest_event() {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};
		assert_ok!(create_asset(collection_hash, b"Asset 1", b"{}"));

		assert_noop!(
			MetaAssetsModule::remove_collection(RuntimeOrigin::signed(ALICE), collection_hash),
			Error::<Test>::SomeAssetsExists
		);
	})
}

#[test]
fn it_should_create_asset_in_collection() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));

		let collection_hash = match get_latest_event() {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};

		assert_ok!(create_asset(collection_hash, b"Asset 1", b"{}"));

		assert_eq!(get_pallet_events().len(), 3);
	})
}
#[test]
fn it_should_not_create_asset_with_invalid_metadata() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));

		let collection_hash = match get_latest_event() {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};

		assert_noop!(
			create_asset(collection_hash, b"Asset 1", b"{lol"),
			Error::<Test>::InvalidJson
		);
	})
}
#[test]
fn it_should_not_create_asset_with_short_name() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));

		let collection_hash = match get_latest_event() {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};

		assert_noop!(
			create_asset(collection_hash, SHORT_ASSET_NAME, b"{}"),
			Error::<Test>::ShortNameProvided
		);
	})
}

#[test]
fn it_should_not_create_collection_with_short_name() {
	ExtBuilder.build().execute_with(|| {
		assert_noop!(
			create_collection(SHORT_COLLECTION_NAME, b"Description xyz", b"{}"),
			Error::<Test>::ShortNameProvided
		);
	})
}

#[test]
fn it_should_not_create_asset_with_long_name() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));

		let collection_hash = match get_latest_event() {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};

		assert_noop!(
			create_asset(collection_hash, LONG_ASSET_NAME, b"{}"),
			Error::<Test>::LongNameProvided
		);
	})
}

#[test]
fn it_should_not_create_collection_with_long_name() {
	ExtBuilder.build().execute_with(|| {
		assert_noop!(
			create_collection(LONG_COLLECTION_NAME, b"Description xyz", b"{}"),
			Error::<Test>::LongNameProvided
		);
	})
}

#[test]
fn it_should_transfer_asset_to_bob() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(create_collection(b"Collection 1", b"Description xyz", b"{}"));

		let collection_hash = match get_latest_event() {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};

		assert_ok!(create_asset(collection_hash, b"Asset 1", b"{}"));

		let asset_hash = match get_latest_event() {
			Event::AssetWasStored { asset_hash, owner: _ } => asset_hash,
			_ => panic!("Unexpected event"),
		};

		assert_ok!(MetaAssetsModule::transfer_asset(
			RuntimeOrigin::signed(ALICE),
			collection_hash.clone(),
			asset_hash.clone(),
			BOB.clone()
		));

		let asset = MetaAssetsModule::assets(collection_hash, asset_hash).unwrap().to_owned();
		assert_eq!(asset.owner, BOB);
	})
}
