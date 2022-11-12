use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_core::{
	offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
	sr25519::{Public, Signature},
};
use std::sync::Arc;

#[test]
fn it_works_for_create_get_account_permissions() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PermissionCaptureModule::create_get_account_permissions(
				Origin::signed(Public::from_raw([1; 32])),
				Public::from_raw([0; 32])
			),
			Error::<Test>::NotCaptureable
		);

		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(Public::from_raw([0; 32])),
			vec![Public::from_raw([1; 32]), Public::from_raw([2; 32]), Public::from_raw([3; 32])],
			2
		));

		assert_noop!(
			PermissionCaptureModule::create_get_account_permissions(
				Origin::signed(Public::from_raw([4; 32])),
				Public::from_raw([0; 32])
			),
			Error::<Test>::MustProposalByFriends
		);
		assert_ok!(PermissionCaptureModule::create_get_account_permissions(
			Origin::signed(Public::from_raw([1; 32])),
			Public::from_raw([0; 32])
		));
		assert_noop!(
			PermissionCaptureModule::create_get_account_permissions(
				Origin::signed(Public::from_raw([1; 32])),
				Public::from_raw([0; 32])
			),
			Error::<Test>::ProposalAlreadyCreated
		);

		assert_eq!(
			ActiveCaptures::<Test>::get(Public::from_raw([0; 32])),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: Default::default(),

				execute_proposal_id: 0,
				cancel_proposal_id: Default::default(),
				capture_statue: CaptureStatue::Processing,
			})
		);

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: Public::from_raw([1; 32]),
				capture_owner: Public::from_raw([0; 32]),

				approve_votes: Default::default(),
				deny_votes: Default::default(),

				statue: ProposalStatue::Processing,
			})
		);
	});
}

#[test]
fn it_works_for_cancel_get_account_permissions() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PermissionCaptureModule::cancel_get_account_permissions(
				Origin::signed(Public::from_raw([1; 32])),
				Public::from_raw([0; 32])
			),
			Error::<Test>::NotCaptureable
		);

		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(Public::from_raw([0; 32])),
			vec![Public::from_raw([1; 32]), Public::from_raw([2; 32]), Public::from_raw([3; 32])],
			2
		));

		assert_noop!(
			PermissionCaptureModule::cancel_get_account_permissions(
				Origin::signed(Public::from_raw([1; 32])),
				Public::from_raw([0; 32])
			),
			Error::<Test>::NotStarted
		);

		assert_ok!(PermissionCaptureModule::create_get_account_permissions(
			Origin::signed(Public::from_raw([1; 32])),
			Public::from_raw([0; 32])
		));

		assert_noop!(
			PermissionCaptureModule::cancel_get_account_permissions(
				Origin::signed(Public::from_raw([5; 32])),
				Public::from_raw([0; 32])
			),
			Error::<Test>::MustProposalByFriends
		);

		assert_ok!(PermissionCaptureModule::cancel_get_account_permissions(
			Origin::signed(Public::from_raw([1; 32])),
			Public::from_raw([0; 32])
		));

		assert_eq!(
			ActiveCaptures::<Test>::get(Public::from_raw([0; 32])),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: Default::default(),

				execute_proposal_id: 0,
				cancel_proposal_id: Some(1),
				capture_statue: CaptureStatue::Processing,
			})
		);

		assert_eq!(
			Proposals::<Test>::get(1),
			Some(Proposal {
				proposal_type: ProposalType::Cancel,
				proposer: Public::from_raw([1; 32]),
				capture_owner: Public::from_raw([0; 32]),

				approve_votes: Default::default(),
				deny_votes: Default::default(),

				statue: ProposalStatue::Processing,
			})
		);
	});
}

#[test]
fn it_works_for_vote_pass() {
	new_test_ext().execute_with(|| {
		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(Public::from_raw([0; 32])),
			vec![Public::from_raw([1; 32]), Public::from_raw([2; 32]), Public::from_raw([3; 32])],
			2
		));

		assert_ok!(PermissionCaptureModule::create_get_account_permissions(
			Origin::signed(Public::from_raw([1; 32])),
			Public::from_raw([0; 32])
		));

		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(Public::from_raw([5; 32])), 0, 0),
			Error::<Test>::MustVoteByFriends
		);

		assert_eq!(MapPermissionTaken::<Test>::get(Public::from_raw([0; 32])), None);

		// first vote
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(Public::from_raw([1; 32])), 0, 0));

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: Public::from_raw([1; 32]),
				capture_owner: Public::from_raw([0; 32]),
				approve_votes: 1,
				deny_votes: Default::default(),
				statue: ProposalStatue::Processing,
			})
		);

		let bounded_friends: FriendsOf<Test> = match vec![Public::from_raw([1; 32])]
			.clone()
			.try_into()
			.map_err(|()| Error::<Test>::MaxFriends)
		{
			Ok(v) => v,
			_ => Default::default(),
		};
		assert_eq!(
			ActiveCaptures::<Test>::get(Public::from_raw([0; 32])),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: bounded_friends,
				friends_cancel: Default::default(),
				execute_proposal_id: 0,
				cancel_proposal_id: Default::default(),
				capture_statue: CaptureStatue::Processing,
			})
		);

		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(Public::from_raw([1; 32])), 0, 0),
			Error::<Test>::AlreadyApproved
		);

		// second votes
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(Public::from_raw([2; 32])), 0, 0));

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: Public::from_raw([1; 32]),
				capture_owner: Public::from_raw([0; 32]),
				approve_votes: 2,
				deny_votes: Default::default(),
				statue: ProposalStatue::End,
			})
		);

		let bounded_friends: FriendsOf<Test> =
			match vec![Public::from_raw([1; 32]), Public::from_raw([2; 32])]
				.clone()
				.try_into()
				.map_err(|()| Error::<Test>::MaxFriends)
			{
				Ok(v) => v,
				_ => Default::default(),
			};
		assert_eq!(
			ActiveCaptures::<Test>::get(Public::from_raw([0; 32])),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: bounded_friends,
				friends_cancel: Default::default(),
				execute_proposal_id: 0,
				cancel_proposal_id: Default::default(),
				capture_statue: CaptureStatue::PermissionTaken,
			})
		);

		assert_eq!(MapPermissionTaken::<Test>::get(Public::from_raw([0; 32])), Some(()));

		// third votes
		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(Public::from_raw([3; 32])), 0, 0),
			Error::<Test>::ProposalMustProcessing
		);
	});
}

#[test]
fn it_works_for_vote_cancel() {
	new_test_ext().execute_with(|| {
		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(Public::from_raw([0; 32])),
			vec![Public::from_raw([1; 32]), Public::from_raw([2; 32]), Public::from_raw([3; 32])],
			2
		));

		assert_ok!(PermissionCaptureModule::create_get_account_permissions(
			Origin::signed(Public::from_raw([1; 32])),
			Public::from_raw([0; 32])
		));
		assert_ok!(PermissionCaptureModule::cancel_get_account_permissions(
			Origin::signed(Public::from_raw([1; 32])),
			Public::from_raw([0; 32])
		));

		// first vote
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(Public::from_raw([1; 32])), 1, 0));

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: Public::from_raw([1; 32]),
				capture_owner: Public::from_raw([0; 32]),
				approve_votes: 0,
				deny_votes: Default::default(),
				statue: ProposalStatue::Processing,
			})
		);

		assert_eq!(
			Proposals::<Test>::get(1),
			Some(Proposal {
				proposal_type: ProposalType::Cancel,
				proposer: Public::from_raw([1; 32]),
				capture_owner: Public::from_raw([0; 32]),
				approve_votes: 1,
				deny_votes: Default::default(),
				statue: ProposalStatue::Processing,
			})
		);

		let bounded_friends: FriendsOf<Test> = match vec![Public::from_raw([1; 32])]
			.clone()
			.try_into()
			.map_err(|()| Error::<Test>::MaxFriends)
		{
			Ok(v) => v,
			_ => Default::default(),
		};
		assert_eq!(
			ActiveCaptures::<Test>::get(Public::from_raw([0; 32])),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: bounded_friends,
				execute_proposal_id: 0,
				cancel_proposal_id: Some(1),
				capture_statue: CaptureStatue::Processing,
			})
		);

		// second votes
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(Public::from_raw([2; 32])), 1, 0));

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: Public::from_raw([1; 32]),
				capture_owner: Public::from_raw([0; 32]),
				approve_votes: 0,
				deny_votes: Default::default(),
				statue: ProposalStatue::End,
			})
		);
		assert_eq!(
			Proposals::<Test>::get(1),
			Some(Proposal {
				proposal_type: ProposalType::Cancel,
				proposer: Public::from_raw([1; 32]),
				capture_owner: Public::from_raw([0; 32]),
				approve_votes: 2,
				deny_votes: Default::default(),
				statue: ProposalStatue::End,
			})
		);

		let bounded_friends: FriendsOf<Test> =
			match vec![Public::from_raw([1; 32]), Public::from_raw([2; 32])]
				.clone()
				.try_into()
				.map_err(|()| Error::<Test>::MaxFriends)
			{
				Ok(v) => v,
				_ => Default::default(),
			};
		assert_eq!(
			ActiveCaptures::<Test>::get(Public::from_raw([0; 32])),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: bounded_friends,
				execute_proposal_id: 0,
				cancel_proposal_id: Some(1),
				capture_statue: CaptureStatue::Canceled,
			})
		);

		// third votes
		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(Public::from_raw([3; 32])), 1, 0),
			Error::<Test>::ProposalMustProcessing
		);
	});
}

#[test]
fn it_works_for_create_capture_config() {
	new_test_ext().execute_with(|| {
		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(Public::from_raw([0; 32])),
			vec![Public::from_raw([1; 32]), Public::from_raw([2; 32]), Public::from_raw([3; 32])],
			2
		));

		let bounded_friends: FriendsOf<Test> = match vec![
			Public::from_raw([1; 32]),
			Public::from_raw([2; 32]),
			Public::from_raw([3; 32]),
		]
		.clone()
		.try_into()
		.map_err(|()| Error::<Test>::MaxFriends)
		{
			Ok(v) => v,
			_ => Default::default(),
		};
		assert_eq!(
			Captureable::<Test>::get(Public::from_raw([0; 32])),
			Some(CaptureConfig { friends: bounded_friends, threshold: 2 })
		);
	});
}

#[test]
fn it_works_for_operational_voting_pass() {
	new_test_ext().execute_with(|| {
		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(Public::from_raw([0; 32])),
			vec![Public::from_raw([1; 32]), Public::from_raw([2; 32]), Public::from_raw([3; 32])],
			2
		));

		assert_ok!(PermissionCaptureModule::create_get_account_permissions(
			Origin::signed(Public::from_raw([1; 32])),
			Public::from_raw([0; 32])
		));

		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(Public::from_raw([5; 32])), 0, 0),
			Error::<Test>::MustVoteByFriends
		);

		assert_eq!(MapPermissionTaken::<Test>::get(Public::from_raw([0; 32])), None);

		// first vote
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(Public::from_raw([1; 32])), 0, 0));
		// second votes
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(Public::from_raw([2; 32])), 0, 0));

		assert_eq!(MapPermissionTaken::<Test>::get(Public::from_raw([0; 32])), Some(()));
	});
}

#[test]
fn it_works_for_operational_voting_cancel() {
	new_test_ext().execute_with(|| {});
}
