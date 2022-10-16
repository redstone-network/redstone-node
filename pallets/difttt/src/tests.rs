use super::*;
use crate::mock::*;

use frame_support::{assert_noop, assert_ok, traits::ConstU32, BoundedVec};
use frame_system::Call;

use mock::{Event, *};
use sp_core::{
	offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
	sr25519::{Public, Signature},
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
		let transfer_protect = Triger::TransferProtect(10, 10000, 10);

		let singer = Public::from_raw([0; 32]);

		// assert create trigger
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), timer));

		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), schedule));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), price_gt));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), price_lt));
		assert_ok!(DiftttModule::create_triger(Origin::signed(singer), transfer_protect));

		// assert storage
		assert_eq!(MapTriger::<Test>::get(0), Some(timer));
		assert_eq!(MapTriger::<Test>::get(1), Some(schedule));
		assert_eq!(MapTriger::<Test>::get(2), Some(price_gt));
		assert_eq!(MapTriger::<Test>::get(3), Some(price_lt));

		assert_eq!(TrigerOwner::<Test>::get(singer, 0), Some(()));
		assert_eq!(TrigerOwner::<Test>::get(singer, 1), Some(()));
		assert_eq!(TrigerOwner::<Test>::get(singer, 2), Some(()));
		assert_eq!(TrigerOwner::<Test>::get(singer, 3), Some(()));

		assert_eq!(NextTrigerId::<Test>::take(), Some(5));

		// assert protect storage
		assert_eq!(AmountLimit::<Test>::take(), Some(10000));
		assert_eq!(TxBlockLimit::<Test>::take(), Some(10));

		// assert event
		println!("_______{},{:?}", 0, timer);
		System::assert_has_event(Event::DiftttModule(crate::Event::TrigerCreated(0, timer)));
		System::assert_has_event(Event::DiftttModule(crate::Event::TrigerCreated(1, schedule)));
		System::assert_has_event(Event::DiftttModule(crate::Event::TrigerCreated(2, price_gt)));
		System::assert_has_event(Event::DiftttModule(crate::Event::TrigerCreated(3, price_lt)));
	})
}

#[test]
fn create_action_should_work() {
	new_test_ext().execute_with(|| {
		// construct instance of params
		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();

		let a_u8_const32: BoundedVec<u8, ConstU32<256>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const128: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();

		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		let oracle: Action<Public> = Action::Slack(a_u8_const32, b_u8_const128);

		let signer = Public::from_raw([0; 32]);

		// assert create action
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token.clone()));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), oracle.clone()));

		// assert storage
		assert_eq!(MapAction::<Test>::get(0), Some(mail_with_token.clone()));
		assert_eq!(MapAction::<Test>::get(1), Some(oracle.clone()));

		assert_eq!(ActionOwner::<Test>::get(signer, 0), Some(()));
		assert_eq!(ActionOwner::<Test>::get(signer, 1), Some(()));

		assert_eq!(NextActionId::<Test>::take(), Some(2));

		// assert event
		System::assert_has_event(Event::DiftttModule(crate::Event::ActionCreated(
			0,
			mail_with_token,
		)));
		System::assert_has_event(Event::DiftttModule(crate::Event::ActionCreated(1, oracle)));
	});
}

#[test]
fn create_recipe_should_work() {
	new_test_ext().execute_with(|| {
		// construct instance of params
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

		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token.clone()));
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));

		let recipe = Recipe {
			triger_id: 0,
			action_id: 0,
			enable: true,
			is_forever: true,
			times: 0,
			max_times: 1,
			done: false,
			last_triger_timestamp: 0,
		};

		// assert storage
		assert_eq!(MapTriger::<Test>::get(0), Some(timer));
		assert_eq!(MapAction::<Test>::get(0), Some(mail_with_token));
		assert_eq!(MapRecipe::<Test>::get(0), Some(recipe.clone()));
		assert_eq!(RecipeOwner::<Test>::get(signer, 0), Some(()));
		assert_eq!(NextRecipeId::<Test>::take(), Some(1));

		// assert event
		System::assert_has_event(Event::DiftttModule(crate::Event::RecipeCreated(0, recipe)));
	})
}
#[test]
fn create_recipe_will_fail_when_trigger_id_not_exist() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let timer = Triger::Timer(1, 1);

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
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));

		assert_noop!(
			DiftttModule::create_recipe(Origin::signed(signer), 1, 0),
			Error::<Test>::TrigerIdNotExist
		);
	});
}

#[test]
fn create_recipe_will_fail_when_action_id_not_exist() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);

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
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));

		assert_noop!(
			DiftttModule::create_recipe(Origin::signed(signer), 0, 1),
			Error::<Test>::ActionIdNotExist
		);
	});
}
#[test]
fn turn_on_recipe_should_work() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		let recipe = Recipe {
			triger_id: 0,
			action_id: 0,
			enable: false,
			is_forever: true,
			times: 0,
			max_times: 1,
			done: false,
			last_triger_timestamp: 0,
		};

		let recipe1 = Recipe {
			triger_id: 0,
			action_id: 0,
			enable: true,
			is_forever: true,
			times: 0,
			max_times: 1,
			done: false,
			last_triger_timestamp: 0,
		};

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token.clone()));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));
		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(signer), 0));

		// assert storage
		assert_eq!(MapTriger::<Test>::get(0), Some(timer));
		assert_eq!(MapAction::<Test>::get(0), Some(mail_with_token));
		assert_eq!(MapRecipe::<Test>::get(0), Some(recipe.clone()));

		// assert turn on recipe
		assert_ok!(DiftttModule::turn_on_recipe(Origin::signed(signer), 0));
		assert_eq!(MapRecipe::<Test>::get(0), Some(recipe1));

		// assert event
		System::assert_has_event(Event::DiftttModule(crate::Event::RecipeTurnOned(0)));
	});
}

#[test]
fn turn_on_recipe_will_fail_when_recipe_id_not_exist() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));
		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(signer), 0));

		// recipe id 1 is not exist
		assert_noop!(
			DiftttModule::turn_on_recipe(Origin::signed(signer), 1),
			Error::<Test>::RecipeIdNotExist
		);
	});
}

#[test]

fn turn_on_recipe_will_fail_when_signer_not_owner() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);
		let signer1 = Public::from_raw([1; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));
		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(signer), 0));

		// recipe id 1 is not exist
		assert_noop!(
			DiftttModule::turn_on_recipe(Origin::signed(signer1), 0),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn turn_off_recipe_should_work() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		let recipe = Recipe {
			triger_id: 0,
			action_id: 0,
			enable: true,
			is_forever: true,
			times: 0,
			max_times: 1,
			done: false,
			last_triger_timestamp: 0,
		};
		let recipe1 = Recipe {
			triger_id: 0,
			action_id: 0,
			enable: false,
			is_forever: true,
			times: 0,
			max_times: 1,
			done: false,
			last_triger_timestamp: 0,
		};

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token.clone()));

		// assert storage
		assert_eq!(MapTriger::<Test>::get(0), Some(timer));
		assert_eq!(MapAction::<Test>::get(0), Some(mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));
		assert_eq!(MapRecipe::<Test>::get(0), Some(recipe));

		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(signer), 0));
		assert_eq!(MapRecipe::<Test>::get(0), Some(recipe1));

		// assert event
		System::assert_has_event(Event::DiftttModule(crate::Event::RecipeTurnOffed(0)));
	});
}

#[test]
fn turn_off_recipe_will_fail_when_recipe_id_not_exist() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));

		// recipe id 1 is not exist
		assert_noop!(
			DiftttModule::turn_off_recipe(Origin::signed(signer), 1),
			Error::<Test>::RecipeIdNotExist
		);
	});
}

#[test]

fn turn_off_recipe_will_fail_when_signer_not_owner() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);
		let signer1 = Public::from_raw([1; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));

		// recipe id 1 is not exist
		assert_noop!(
			DiftttModule::turn_off_recipe(Origin::signed(signer1), 0),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn del_recipe_should_work() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		// assert create trigger, action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(
			Origin::signed(Public::from_raw([0; 32])),
			mail_with_token
		));
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));

		// delete recipe
		assert_ok!(DiftttModule::del_recipe(Origin::signed(signer), 0));

		// assert recipe is not exist
		assert_noop!(
			DiftttModule::del_recipe(Origin::signed(signer), 0),
			Error::<Test>::RecipeIdNotExist
		);

		// assert event
		System::assert_has_event(Event::DiftttModule(crate::Event::RecipeRemoved(0)));
	});
}

#[test]
fn delete_recipe_will_fail_when_recipe_id_not_exist() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));

		// recipe id 1 is not exist
		assert_noop!(
			DiftttModule::del_recipe(Origin::signed(signer), 1),
			Error::<Test>::RecipeIdNotExist
		);
	});
}

#[test]

fn delete_recipe_will_fail_when_signer_not_owner() {
	new_test_ext().execute_with(|| {
		let timer = Triger::Timer(1, 1);
		let signer = Public::from_raw([0; 32]);
		let signer1 = Public::from_raw([1; 32]);

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));
		assert_ok!(DiftttModule::turn_off_recipe(Origin::signed(signer), 0));

		// recipe id 1 is not exist
		assert_noop!(DiftttModule::del_recipe(Origin::signed(signer1), 0), Error::<Test>::NotOwner);
	});
}

#[test]
fn submit_recipe_done_with_signed_payload_should_work() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let signer1 = Signature::from_raw([0; 64]);

		let timer = Triger::Timer(1, 1);

		type BlockNumber = u64;

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));

		let block_number: BlockNumber = 64; //区块问题

		let recipe_done_payload: RecipeDonePayload<Public, BlockNumber> =
			RecipeDonePayload { block_number, recipe_id: 0, public: signer };

		assert_ok!(DiftttModule::submit_recipe_done_with_signed_payload(
			Origin::none(),
			recipe_done_payload,
			signer1
		));

		System::assert_has_event(Event::DiftttModule(crate::Event::RecipeDone(0)));
	});
}

#[test]
fn submit_recipe_done_with_signed_payload_will_fail_when_recipe_id_not_exist() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let signer1 = Signature::from_raw([0; 64]);

		type BlockNumber = u64;

		let block_number: BlockNumber = 64; //区块问题

		let recipe_done_payload: RecipeDonePayload<Public, BlockNumber> =
			RecipeDonePayload { block_number, recipe_id: 0, public: signer };

		// recipe id 0 is not exist
		assert_noop!(
			DiftttModule::submit_recipe_done_with_signed_payload(
				Origin::none(),
				recipe_done_payload,
				signer1
			),
			Error::<Test>::RecipeIdNotExist
		);
	});
}

#[test]
fn submit_recipe_triger_times_with_signed_payload_should_work() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let signer1 = Signature::from_raw([0; 64]);

		let timer = Triger::Timer(1, 1);

		type BlockNumber = u64;

		let a_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let b_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let c_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let d_u8_const128: BoundedVec<u8, ConstU32<128>> = vec![1, 2, 3].try_into().unwrap();
		let e_u8_const256: BoundedVec<u8, ConstU32<256>> = vec![4, 5, 6].try_into().unwrap();
		let mail_with_token: Action<Public> = Action::MailWithToken(
			a_u8_const128,
			b_u8_const256,
			c_u8_const128,
			d_u8_const128,
			e_u8_const256,
		);

		// assert create trigger , action and recipe
		assert_ok!(DiftttModule::create_triger(Origin::signed(signer), timer));
		assert_ok!(DiftttModule::create_action(Origin::signed(signer), mail_with_token));

		// assert create and turn off recipe
		assert_ok!(DiftttModule::create_recipe(Origin::signed(signer), 0, 0));

		let block_number: BlockNumber = 64; //区块问题
		let times: u64 = 10;
		let timestamp: u64 = 10;

		let recipe_times_payload: RecipeTimesPayload<Public, BlockNumber> =
			RecipeTimesPayload { block_number, recipe_id: 0, times, timestamp, public: signer };

		assert_ok!(DiftttModule::submit_recipe_triger_times_with_signed_payload(
			Origin::none(),
			recipe_times_payload,
			signer1
		));

		// assert event
		System::assert_has_event(Event::DiftttModule(crate::Event::RecipeTrigerTimesUpdated(
			0, times, timestamp,
		)));
	});
}

#[test]
fn submit_recipe_triger_times_with_signed_payload_will_fail_when_recipe_not_exist() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let signer1 = Signature::from_raw([0; 64]);

		type BlockNumber = u64;
		let block_number: BlockNumber = 64; //区块问题
		let times: u64 = 10;
		let timestamp: u64 = 10;

		let recipe_times_payload: RecipeTimesPayload<Public, BlockNumber> =
			RecipeTimesPayload { block_number, recipe_id: 0, times, timestamp, public: signer };

		// recipe id 0 is not exist
		assert_noop!(
			DiftttModule::submit_recipe_triger_times_with_signed_payload(
				Origin::none(),
				recipe_times_payload,
				signer1
			),
			Error::<Test>::RecipeIdNotExist
		);
	});
}
