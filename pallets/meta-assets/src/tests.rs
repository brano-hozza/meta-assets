use super::*;
use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn it_creates_collection() {
	ExtBuilder.build().execute_with(|| {
		let collection_name: BoundedString<Test> = b"Collection 1".to_vec().try_into().unwrap();

		let description: BoundedString<Test> = b"Description".to_vec().try_into().unwrap();

		let schema: BoundedJson<Test> = b"{}".to_vec().try_into().unwrap();

		let collection = MetaAssetsModule::create_collection(
			RuntimeOrigin::signed(ALICE),
			collection_name.clone(),
			description.clone(),
			schema.clone(),
		);
		assert_ok!(collection);

		let events = only_pallet_events();

		assert_eq!(events.len(), 1);
	})
}

#[test]
fn it_removes_collection() {
	ExtBuilder.build().execute_with(|| {
		let collection_name: BoundedString<Test> = b"Collection 1".to_vec().try_into().unwrap();

		let description: BoundedString<Test> = b"Description".to_vec().try_into().unwrap();

		let schema: BoundedJson<Test> = b"{}".to_vec().try_into().unwrap();

		let collection = MetaAssetsModule::create_collection(
			RuntimeOrigin::signed(ALICE),
			collection_name.clone(),
			description.clone(),
			schema.clone(),
		);
		assert_ok!(collection);

		let events = only_pallet_events();

		assert_eq!(events.len(), 1);

		let created_event = events[0].clone();

		let collection_hash = match created_event {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};

		let remove_collection =
			MetaAssetsModule::remove_collection(RuntimeOrigin::signed(ALICE), collection_hash);

		assert_ok!(remove_collection);

		let events = only_pallet_events();

		assert_eq!(events.len(), 2);
	})
}

#[test]
fn it_creates_asset_in_collection() {
	ExtBuilder.build().execute_with(|| {
		let collection_name: BoundedString<Test> = b"Collection 1".to_vec().try_into().unwrap();

		let description: BoundedString<Test> = b"Description".to_vec().try_into().unwrap();

		let schema: BoundedJson<Test> = b"{}".to_vec().try_into().unwrap();

		let collection = MetaAssetsModule::create_collection(
			RuntimeOrigin::signed(ALICE),
			collection_name.clone(),
			description.clone(),
			schema.clone(),
		);
		assert_ok!(collection);

		let events = only_pallet_events();

		assert_eq!(events.len(), 1);

		let created_event = events[0].clone();

		let collection_hash = match created_event {
			Event::CollectionCreated { collection_hash, owner: _ } => collection_hash,
			_ => panic!("Unexpected event"),
		};

		let asset_name: BoundedString<Test> = b"Asset 1".to_vec().try_into().unwrap();

		let asset_meta: BoundedJson<Test> = b"{}".to_vec().try_into().unwrap();

		let asset = MetaAssetsModule::add_asset(
			RuntimeOrigin::signed(ALICE),
			asset_name.clone(),
			collection_hash.clone(),
			asset_meta.clone(),
		);

		assert_ok!(asset);

		let events = only_pallet_events();

		assert_eq!(events.len(), 3);
	})
}
