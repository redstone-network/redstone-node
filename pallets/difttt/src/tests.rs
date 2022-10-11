use super::*;
use crate::mock::*;
use frame_support::{assert_ok, traits::ConstU32, BoundedVec};
use frame_system::Call;

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

		// assert create action
		let singer = Public::from_raw([0; 32]);

		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), timer));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), schedule));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), price_gt));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), price_lt));

		// check storage
		assert_eq!(
			MapTriger::<Test>::get(0),
			Some(timer) // TrigerOwner::<T>::insert(user, triger_id, ());		            // NextTrigerId::<T>::put(triger_id.saturating_add(One::one()));
		);

		// MapTriger::<T>::insert(triger_id, triger.clone());
		// TrigerOwner::<T>::insert(user, triger_id, ());
		// NextTrigerId::<T>::put(triger_id.saturating_add(One::one()));

		// assert_eq!(
		// 	TrigerOwner::<Test>::get(&singer, 0),
		// 	Some(timer) // TrigerOwner::<T>::insert(user, triger_id, ());		            // NextTrigerId::<T>::put(triger_id.saturating_add(One::one()));
		// );
	})

	// #[test]
	// fn create_action_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let mail_with_token = Action::MailWithToken(
	// 			a_u8_const128,
	// 			b_u8_const256,
	// 			c_u8_const128,
	// 			d_u8_const128,
	// 			e_u8_const256,
	// 		);
	// 		let a_u8_const32: BoundedVec<u8, ConstU32<256>> = vec![1, 2, 3].try_into().unwrap();
	// 		let b_u8_const128: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();

	// 		let oracle = Action::Slack(a_u8_const32, b_u8_const128);
	// 		// Dispatch a signed extrinsic.
	// 		assert_ok!(DiftttModule::create_action(
	// 			Origin::signed(Public::from_raw([0; 32]))),
	// 			mail_with_token
	// 		)); //枚举实例一，通过
	// 		assert_ok!(DiftttModule::create_action(Origin::signed(Public::from_raw([0; 32])), oracle));
	// 		//枚举实例二，通过
	// 	});
	// }

	// #[test]
	// fn create_recipe_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		let timer = Triger::Timer(1, 1);
	// 		assert_ok!(DiftttModule::create_triger(Origin::signed(Public::from_raw([0; 32])), timer)); //枚举实例一，通过

	// 		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let mail_with_token = Action::MailWithToken(
	// 			a_u8_const128,
	// 			b_u8_const256,
	// 			c_u8_const128,
	// 			d_u8_const128,
	// 			e_u8_const256,
	// 		);
	// 		assert_ok!(DiftttModule::create_action(
	// 			Origin::signed(Public::from_raw([0; 32])),
	// 			mail_with_token
	// 		)); //枚举实例一，通过

	// 		assert_ok!(DiftttModule::create_recipe(Origin::signed(Public::from_raw([0; 32])), 0, 0));
	// 		//测试失败，可能是参数设置不正确
	// 	});
	// }

	// #[test]
	// fn turn_on_recipe_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		let timer = Triger::Timer(1, 1);
	// 		assert_ok!(DiftttModule::create_triger(Origin::signed(Public::from_raw([0; 32])), timer)); //枚举实例一，通过

	// 		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let mail_with_token = Action::MailWithToken(
	// 			a_u8_const128,
	// 			b_u8_const256,
	// 			c_u8_const128,
	// 			d_u8_const128,
	// 			e_u8_const256,
	// 		);
	// 		assert_ok!(DiftttModule::create_action(
	// 			Origin::signed(Public::from_raw([0; 32])),
	// 			mail_with_token
	// 		)); //枚举实例一，通过

	// 		assert_ok!(DiftttModule::create_recipe(Origin::signed(Public::from_raw([0; 32])), 0, 0)); //测试失败，可能是参数设置不正确

	// 		assert_ok!(DiftttModule::turn_on_recipe(Origin::signed(Public::from_raw([0; 32])), 0));
	// 		//测试失败，可能是参数设置不正确
	// 	});
	// }

	// #[test]
	// fn turn_off_recipe_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		let timer = Triger::Timer(1, 1);
	// 		assert_ok!(DiftttModule::create_triger(Origin::signed(Public::from_raw([0; 32])), timer)); //枚举实例一，通过

	// 		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let mail_with_token = Action::MailWithToken(
	// 			a_u8_const128,
	// 			b_u8_const256,
	// 			c_u8_const128,
	// 			d_u8_const128,
	// 			e_u8_const256,
	// 		);
	// 		assert_ok!(DiftttModule::create_action(
	// 			Origin::signed(Public::from_raw([0; 32])),
	// 			mail_with_token
	// 		)); //枚举实例一，通过

	// 		assert_ok!(DiftttModule::create_recipe(Origin::signed(Public::from_raw([0; 32])), 0, 0)); //测试失败，可能是参数设置不正确

	// 		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(Public::from_raw([0; 32])), 0));
	// 		//测试失败，可能是参数设置不正确
	// 	});
	// }

	// #[test]
	// fn del_recipe_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		let timer = Triger::Timer(1, 1);
	// 		assert_ok!(DiftttModule::create_triger(Origin::signed(Public::from_raw([0; 32])), timer)); //枚举实例一，通过

	// 		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
	// 		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
	// 		let mail_with_token = Action::MailWithToken(
	// 			a_u8_const128,
	// 			b_u8_const256,
	// 			c_u8_const128,
	// 			d_u8_const128,
	// 			e_u8_const256,
	// 		);
	// 		assert_ok!(DiftttModule::create_action(
	// 			Origin::signed(Public::from_raw([0; 32])),
	// 			mail_with_token
	// 		)); //枚举实例一，通过

	// 		assert_ok!(DiftttModule::create_recipe(Origin::signed(Public::from_raw([0; 32])), 0, 0)); //测试失败，可能是参数设置不正确

	// 		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(Public::from_raw([0; 32])), 0));
	// 		//测试失败，可能是参数设置不正确
	// 	});
}

// #[test]
// fn set_recipe_done_unsigned_should_work() {}
