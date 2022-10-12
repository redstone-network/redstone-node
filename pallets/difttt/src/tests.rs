use super::*;
// use crate::mock::*;
use frame_support::{assert_ok, traits::ConstU32, BoundedVec};
use frame_system::Call;

use mock::{new_test_ext, DiftttModule, Event as TestEvent, Origin, System, Test};
use sp_core::{
	offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
	sr25519::Public,
};
use sp_keystore::{testing::KeyStore, KeystoreExt};
use std::sync::Arc;

#[test]
fn create_trigger_should_work() {
	new_test_ext().execute_with(|| {
		// construct instance of params
		let timer = Triger::Timer(1, 1);
		let schedule = Triger::Schedule(2, 2);
		let price_gt = Triger::PriceGT(3, 3);
		let price_lt = Triger::PriceLT(4, 4);

		let singer = Public::from_raw([0; 32]);

		// assert create trigger
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), timer));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), schedule));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), price_gt));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), price_lt));

		// check storage
		assert_eq!(MapTriger::<Test>::get(0), Some(timer));
		assert_eq!(MapTriger::<Test>::get(1), Some(schedule));
		assert_eq!(MapTriger::<Test>::get(2), Some(price_gt));
		assert_eq!(MapTriger::<Test>::get(3), Some(price_lt));

		assert_eq!(TrigerOwner::<Test>::get(singer, 0), Some(()));
		assert_eq!(TrigerOwner::<Test>::get(singer, 1), Some(()));
		assert_eq!(TrigerOwner::<Test>::get(singer, 2), Some(()));
		assert_eq!(TrigerOwner::<Test>::get(singer, 3), Some(()));

		assert_eq!(NextTrigerId::<Test>::take(), Some(4));

		//check event
		// System::assert_has_event(TestEvent::DiftttModule(Event::TrigerCreated(0, timer)));
		// System::assert_has_event(TestEvent::DiftttModule(Event::TrigerCreated(1, schedule)));
		// System::assert_has_event(TestEvent::DiftttModule(Event::TrigerCreated(2, price_gt)));
		// System::assert_has_event(TestEvent::DiftttModule(Event::TrigerCreated(3, price_lt)));
	})
}

#[test]
fn create_action_should_work() {
	new_test_ext().execute_with(|| {
		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);
		let a_u8_const32: BoundedVec<u8, ConstU32<256>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const128: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();

		let oracle = Action::Slack(a_u8_const32, b_u8_const128);
		let signer = Public::from_raw([0; 32]);
		// Dispatch a signed extrinsic.
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), oracle));
	})
}

#[test]
fn create_recipe_should_work() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);
		assert_ok!(DiftttModule::create_triger(Origin::signed(Public::from_raw([0; 32])), timer));

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));
	})
}

#[test]
fn turn_on_recipe_should_work() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);
		assert_ok!(DiftttModule::create_triger(Origin::signed(Public::from_raw([0; 32])), timer));
		//枚举实例一，通过

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));
		assert_ok!(DiftttModule::turn_on_recipe(Origin::signed(signer), 0));
	});
}

#[test]
fn turn_off_recipe_should_work() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));
		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(signer), 0));
	});
}

#[test]
fn del_recipe_should_work() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);
		assert_ok!(DiftttModule::create_action(
			Origin::signed(Public::from_raw([0; 32])),
			mail_with_token
		));

		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));

		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(signer), 0));
	})
}

#[test]
fn set_recipe_done_unsigned_should_work() {}
