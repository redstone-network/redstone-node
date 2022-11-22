use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};

use mock::Event;
use sp_core::sr25519::Public;

#[test]
fn set_transfer_limit_should_work() {
	new_test_ext().execute_with(|| {
		let amount_limit = TransferLimit::AmountLimit(1000);
		let times_limit = TransferLimit::FrequencyLimit(5, 100);
		let signer = Public::from_raw([0; 32]);

		let block_number = System::block_number();

		// assert
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), times_limit));

		// assert MapTransferLimit
		assert_eq!(MapTransferLimit::<Test>::get(0), Some((block_number, amount_limit)));
		assert_eq!(MapTransferLimit::<Test>::get(1), Some((block_number, times_limit)));

		// assert transfer limit owner
		assert_eq!(TransferLimitOwner::<Test>::get(signer, 0), Some(amount_limit));
		assert_eq!(TransferLimitOwner::<Test>::get(signer, 1), Some(times_limit));

		// assert next transfer_limit_id
		assert_eq!(NextTransferLimitId::<Test>::take(), Some(2));

		// assert successful events
		System::assert_has_event(Event::DefenseModule(crate::Event::TransferAmountLimitSet(
			signer,
			amount_limit,
		)));
		System::assert_has_event(Event::DefenseModule(crate::Event::TransferFrequencyLimitSet(
			signer,
			times_limit,
		)));

		let update_amount_limit = TransferLimit::AmountLimit(2000);
		let update_times_limit = TransferLimit::FrequencyLimit(10, 100);

		// assert
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), update_amount_limit));
		// assert successful events

		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), update_times_limit));

		// assert MapTransferLimit
		assert_eq!(MapTransferLimit::<Test>::get(0), Some((block_number, update_amount_limit)));
		assert_eq!(MapTransferLimit::<Test>::get(1), Some((block_number, update_times_limit)));

		// assert transfer limit owner
		assert_eq!(TransferLimitOwner::<Test>::get(signer, 0), Some(update_amount_limit));
		assert_eq!(TransferLimitOwner::<Test>::get(signer, 1), Some(update_times_limit));

		// assert next transfer_limit_id
		assert_eq!(NextTransferLimitId::<Test>::take(), Some(2));
	});
}

#[test]
fn set_risk_management_should_work() {
	new_test_ext().execute_with(|| {
		let freeze = true;

		let freeze_account_for_some_time = RiskManagement::TimeFreeze(120);
		let freeze_account_forever = RiskManagement::AccountFreeze(freeze);
		let signer = Public::from_raw([0; 32]);

		let block_number = System::block_number();

		// assert
		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));
		assert_ok!(DefenseModule::set_risk_management(
			Origin::signed(signer),
			freeze_account_forever.clone()
		));

		// assert MapTransferLimit
		assert_eq!(
			MapRiskManagement::<Test>::get(0),
			Some((block_number, freeze_account_for_some_time.clone()))
		);
		assert_eq!(
			MapRiskManagement::<Test>::get(1),
			Some((block_number, freeze_account_forever.clone()))
		);

		// assert transfer limit owner
		assert_eq!(
			RiskManagementOwner::<Test>::get(signer, 0),
			Some(freeze_account_for_some_time.clone())
		);
		assert_eq!(
			RiskManagementOwner::<Test>::get(signer, 1),
			Some(freeze_account_forever.clone())
		);

		// assert next transfer_limit_id
		assert_eq!(NextRiskManagementId::<Test>::take(), Some(2));

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

		let amount_limit = TransferLimit::AmountLimit(1000);
		let times_limit = TransferLimit::FrequencyLimit(10, 1000);

		// assert
		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));

		// assert
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 2));

		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), times_limit));
		// assert
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		System::assert_has_event(Event::DefenseModule(crate::Event::TransferSuccess(
			signer, to, 3,
		)));
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
		let to = Public::from_raw([1; 32]);

		let amount_limit = TransferLimit::AmountLimit(50);

		assert_ok!(DefenseModule::set_transfer_limit(Origin::signed(signer), amount_limit));

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 51),
			Error::<Test>::TransferValueTooLarge
		);
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

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 3),
			Error::<Test>::TransferTimesTooMany
		);
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

		System::assert_has_event(Event::DefenseModule(crate::Event::FreezeAccountTemporary(
			signer,
		)));

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

		System::assert_has_event(Event::DefenseModule(crate::Event::TransferSuccess(
			signer, to, 3,
		)));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		System::assert_has_event(Event::DefenseModule(crate::Event::FreezeAccountForever(signer)));

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 3),
			Error::<Test>::AccountHasBeenFrozenForever
		);
	});
}
