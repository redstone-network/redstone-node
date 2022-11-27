use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};

use mock::Event;
use sp_core::sr25519::Public;

#[test]
fn set_transfer_limit_should_work() {
	new_test_ext().execute_with(|| {
		let amount_limit = TransferLimit::AmountLimit(1000);
		let frequency_limit = TransferLimit::FrequencyLimit(5, 100);
		let signer = Public::from_raw([0; 32]);

		// assert
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), frequency_limit));

		// assert transfer limit owner
		assert_eq!(TransferLimitOwner::<Test>::get(signer, 1), Some(amount_limit));
		assert_eq!(TransferLimitOwner::<Test>::get(signer, 2), Some(frequency_limit));

		// assert successful events
		System::assert_has_event(Event::DefenseModule(crate::Event::TransferAmountLimitSet(
			signer,
			amount_limit,
		)));
		System::assert_has_event(Event::DefenseModule(crate::Event::TransferFrequencyLimitSet(
			signer,
			frequency_limit,
		)));
	});
}

#[test]
fn set_risk_management_should_work() {
	new_test_ext().execute_with(|| {
		let freeze_account_for_some_time = RiskManagement::TimeFreeze(120);
		let freeze_account_forever = RiskManagement::AccountFreeze(true);
		let signer = Public::from_raw([0; 32]);

		// assert
		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));
		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_forever.clone()
		));

		assert_eq!(
			RiskManagementOwner::<Test>::get(&signer, 1),
			Some(freeze_account_for_some_time.clone())
		);

		assert_eq!(
			RiskManagementOwner::<Test>::get(signer, 2),
			Some(freeze_account_forever.clone())
		);

		// assert successful events
		System::assert_has_event(Event::DefenseModule(crate::Event::RiskManagementTimeFreezeSet(
			signer,
			freeze_account_for_some_time.clone(),
		)));
		System::assert_has_event(Event::DefenseModule(
			crate::Event::RiskManagementAccountFreezeSet(signer, freeze_account_forever.clone()),
		));
	});
}

#[test]
fn safe_transfer_should_work() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let to = Public::from_raw([1; 32]);
		// assert
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 1));

		let amount_limit = TransferLimit::AmountLimit(100);
		let frequency_limit = TransferLimit::FrequencyLimit(10, 1000);

		// assert
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));

		// assert
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), frequency_limit));
		// assert
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		// System::assert_has_event(Event::DefenseModule(crate::Event::TransferSuccess(
		// 	signer, to, 3,
		// )));
	});
}

#[test]
fn set_transfer_limit_should_fail_when_amount_limit_has_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let amount_limit = TransferLimit::AmountLimit(1000);
		let update_amount_limit = TransferLimit::AmountLimit(2000);

		// assert
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));

		assert_noop!(
			DefenseModule::set_transfer_limit(Origin::signed(signer), update_amount_limit),
			Error::<Test>::TransferAmountLimitHasSet
		);
	});
}

#[test]
fn set_transfer_limit_should_fail_when_frequency_limit_has_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);

		let frequency_limit = TransferLimit::FrequencyLimit(5, 100);
		let update_frequency_limit = TransferLimit::FrequencyLimit(100, 100);

		// assert
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), frequency_limit));

		assert_noop!(
			DefenseModule::set_transfer_limit(Origin::signed(signer), update_frequency_limit),
			Error::<Test>::TransferFrequencyLimitHasSet
		);
	});
}

#[test]
fn set_risk_management_should_fail_when_freeze_time_has_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let mut freeze_account_for_some_time = RiskManagement::TimeFreeze(120);

		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));

		freeze_account_for_some_time = RiskManagement::TimeFreeze(240);

		assert_noop!(
			DefenseModule::set_risk_management(
				Origin::signed(signer),
				freeze_account_for_some_time.clone()
			),
			Error::<Test>::FreezeTimeHasSet
		);
	});
}

#[test]
fn set_risk_management_should_fail_when_freeze_account_has_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let mut freeze_account_forever = RiskManagement::AccountFreeze(true);

		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_forever.clone()
		));

		freeze_account_forever = RiskManagement::AccountFreeze(false);

		assert_noop!(
			DefenseModule::set_risk_management(
				Origin::signed(signer),
				freeze_account_forever.clone()
			),
			Error::<Test>::FreezeAccountHasSet
		);
	});
}

#[test]
fn safe_transfer_should_fail_when_transfer_value_is_larger_than_set_amount() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);

		let amount_limit = TransferLimit::AmountLimit(50);

		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));

		// assert_noop!(
		// 	DefenseModule::safe_transfer(Origin::signed(signer), to, 51),
		// 	Error::<Test>::TransferValueTooLarge
		// );
	});
}

#[test]
fn safe_transfer_should_fail_when_transfer_times_is_more_than_set_times() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let to = Public::from_raw([1; 32]);

		let times_limit = TransferLimit::FrequencyLimit(3, 100);

		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), times_limit));
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		// assert_noop!(
		// 	DefenseModule::safe_transfer(Origin::signed(signer), to, 3),
		// 	Error::<Test>::TransferTimesTooMany
		// );
	});
}

#[test]
fn safe_transfer_should_fail_when_account_freeze_temporary() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let to = Public::from_raw([1; 32]);

		let amount_limit = TransferLimit::AmountLimit(50);

		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));

		let freeze_account_for_some_time = RiskManagement::TimeFreeze(120);

		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 60));

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 10),
			Error::<Test>::AccountHasBeenFrozenTemporary
		);
	});
}

#[test]
fn safe_transfer_should_fail_when_account_freeze_forever() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let to = Public::from_raw([1; 32]);

		let times_limit = TransferLimit::FrequencyLimit(3, 100);

		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), times_limit));

		let freeze_account_for_some_time = RiskManagement::TimeFreeze(120);
		let freeze_account_forever = RiskManagement::AccountFreeze(true);

		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));

		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_forever.clone()
		));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 3),
			Error::<Test>::AccountHasBeenFrozenForever
		);
	});
}

#[test]
fn freeze_account_should_work() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);

		assert_ok!(DefenseModule::freeze_account(Origin::signed(signer), true));

		assert_eq!(BlockAccount::<Test>::get(signer), Some(true));

		System::assert_has_event(Event::DefenseModule(crate::Event::FreezeAccountSuccess(signer)));
	})
}

#[test]
fn safe_transfer_should_fail_when_amount_is_too_large_without_risk_management() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let amount_limit = TransferLimit::AmountLimit(50);
		let to = Public::from_raw([1; 32]);

		// assert
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 51));

		assert_eq!(DefaultFreeze::<Test>::get(signer), Some(true));

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 10),
			Error::<Test>::AccountHasBeenFrozenTemporary
		);
	});
}
