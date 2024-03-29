use crate::{
    mock::*, AdAsset, AdsOf, Config, CurrencyOrAsset, DeadlineOf, EndtimeOf, Error, Metadata,
    SlotOf,
};
use frame_support::{assert_noop, assert_ok, traits::Hooks};
use parami_primitives::constants::DOLLARS;
use parami_traits::Tags;
use sp_core::crypto::AccountId32;
use sp_core::{ByteArray, Pair};
use sp_runtime::traits::Hash;
use sp_runtime::MultiAddress;
use sp_std::collections::btree_map::BTreeMap;

#[test]
fn should_create() {
    new_test_ext().execute_with(|| {
        let tags = vec![
            vec![5u8, 4u8, 3u8, 2u8, 1u8, 0u8],
            vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
        ];

        let mut hashes = BTreeMap::new();
        for tag in &tags {
            let hash = Tag::key(tag);
            hashes.insert(hash, true);
        }

        let metadata = vec![0u8; 64];

        assert_ok!(Ad::create(
            Origin::signed(ALICE),
            tags,
            metadata.clone(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        assert_eq!(<AdsOf<Test>>::get(&DID_ALICE).unwrap().len(), 1);

        let maybe_ad = <Metadata<Test>>::iter().next();
        assert_ne!(maybe_ad, None);

        let (ad, meta) = maybe_ad.unwrap();
        assert_eq!(meta.creator, DID_ALICE);
        assert_eq!(meta.metadata, metadata);
        assert_eq!(meta.reward_rate, 1);
        assert_eq!(meta.created, 0);

        assert_eq!(<EndtimeOf<Test>>::get(&ad), Some(1));

        assert_eq!(<Test as Config>::Tags::tags_of(&ad), hashes);
    });
}

#[test]
fn should_fail_when_min_greater_than_max() {
    new_test_ext().execute_with(|| {
        let tags = vec![
            vec![5u8, 4u8, 3u8, 2u8, 1u8, 0u8],
            vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
        ];

        let mut hashes = BTreeMap::new();
        for tag in &tags {
            let hash = Tag::key(tag);
            hashes.insert(hash, true);
        }

        let metadata = vec![0u8; 64];

        assert_noop!(
            Ad::create(
                Origin::signed(ALICE),
                tags,
                metadata.clone(),
                1,
                1,
                1u128,
                11u128,
                10u128,
                None,
            ),
            Error::<Test>::WrongPayoutSetting
        );
    });
}

#[test]
fn should_fail_when_tag_not_exists() {
    new_test_ext().execute_with(|| {
        let tags = vec![
            vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
            vec![5u8, 4u8, 3u8, 2u8, 1u8, 0u8],
            vec![0u8; 6],
        ];

        assert_noop!(
            Ad::create(
                Origin::signed(ALICE),
                tags,
                [0u8; 64].into(),
                1,
                1,
                1u128,
                0,
                10u128,
                None
            ),
            Error::<Test>::TagNotExists
        );
    });
}

#[test]
fn should_update_reward_rate() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ad::create(
            Origin::signed(ALICE),
            vec![],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        assert_ok!(Ad::update_reward_rate(Origin::signed(ALICE), ad, 2));

        assert_eq!(<Metadata<Test>>::get(&ad).unwrap().reward_rate, 2);
    });
}

#[test]
fn should_fail_when_not_exists_or_not_owned() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ad::update_reward_rate(Origin::signed(ALICE), Default::default(), 2),
            Error::<Test>::NotExists
        );

        assert_ok!(Ad::create(
            Origin::signed(ALICE),
            vec![],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        assert_noop!(
            Ad::update_reward_rate(Origin::signed(BOB), ad, 2),
            Error::<Test>::NotOwnedOrDelegated
        );
    });
}

#[test]
fn should_update_tags() {
    new_test_ext().execute_with(|| {
        let tags = vec![
            vec![5u8, 4u8, 3u8, 2u8, 1u8, 0u8],
            vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
        ];

        let mut hashes = BTreeMap::new();
        for tag in &tags {
            let hash = Tag::key(tag);
            hashes.insert(hash, true);
        }

        assert_ok!(Ad::create(
            Origin::signed(ALICE),
            vec![vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        assert_ok!(Ad::update_tags(Origin::signed(ALICE), ad, tags));

        assert_eq!(<Test as Config>::Tags::tags_of(&ad), hashes);
    });
}

#[test]
fn should_generate_unique_slot_pot() {
    new_test_ext().execute_with(|| {
        let pot1 = Ad::generate_slot_pot(0);
        let pot2 = Ad::generate_slot_pot(1);

        assert_ne!(pot1, pot2);
    });
}

#[test]
fn should_bid_with_fraction() {
    new_test_ext().execute_with(|| {
        // 1. prepare

        let nft = Nft::preferred(DID_ALICE).unwrap();
        let meta = Nft::meta(nft).unwrap();
        let endtime = 43200;

        // ad1
        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![],
            [0u8; 64].into(),
            1,
            endtime,
            1u128,
            0,
            10u128,
            None
        ));

        let ad1 = <Metadata<Test>>::iter_keys().next().unwrap();

        // 2. bob bid for ad1

        let slot = <SlotOf<Test>>::get(nft);
        assert_eq!(slot, None);

        let bob_bid_fraction = 400;

        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad1,
            nft,
            bob_bid_fraction,
            None,
            None
        ));

        // ensure: deadline, slot, remain
        assert_eq!(<EndtimeOf<Test>>::get(&ad1), Some(endtime));
        assert_eq!(<DeadlineOf<Test>>::get(nft, &ad1), Some(endtime));

        let slot = <SlotOf<Test>>::get(nft).unwrap();
        assert_eq!(slot.ad_id, ad1);

        // 3. charlie bid for ad2
        // ad2

        assert_ok!(Ad::create(
            Origin::signed(CHARLIE),
            vec![],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        let ad2 = <Metadata<Test>>::iter_keys().next().unwrap();

        assert_noop!(
            Ad::bid_with_fraction(
                Origin::signed(CHARLIE),
                ad2,
                nft,
                bob_bid_fraction.saturating_mul(120).saturating_div(100),
                None,
                None
            ),
            Error::<Test>::Underbid
        );

        assert_eq!(
            Assets::balance(meta.token_asset_id, CHARLIE),
            CHARLIE_BALANCE
        );
        let charlie_bid_fraction = bob_bid_fraction
            .saturating_mul(120)
            .saturating_div(100)
            .saturating_add(1);
        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(CHARLIE),
            ad2,
            nft,
            charlie_bid_fraction,
            None,
            None
        ));
        assert_eq!(
            Assets::balance(meta.token_asset_id, CHARLIE),
            CHARLIE_BALANCE - charlie_bid_fraction
        );

        let slot = <SlotOf<Test>>::get(nft).unwrap();
        assert_eq!(slot.ad_id, ad2);

        let locked_fraction = Assets::balance(meta.token_asset_id, slot.budget_pot);
        assert_eq!(locked_fraction, charlie_bid_fraction);

        // ensure: deadline, slot, remain

        assert_eq!(<EndtimeOf<Test>>::get(&ad2), Some(1));
        assert_eq!(<DeadlineOf<Test>>::get(nft, &ad1), None);
        assert_eq!(<DeadlineOf<Test>>::get(nft, &ad2), Some(1));
    });
}

#[test]
fn should_fail_to_add_budget_when_fungible_not_same_with_bid() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::force_create(
            Origin::root(),
            9,
            MultiAddress::Id(BOB),
            true,
            1
        ));
        let fungible_id = 9;
        assert_ok!(Assets::mint(
            Origin::signed(BOB),
            fungible_id,
            MultiAddress::Id(BOB),
            1000
        ));

        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        let nft = Nft::preferred(DID_ALICE).unwrap();
        let ad = <Metadata<Test>>::iter_keys().next().unwrap();
        let bob_bid_fraction = 250;

        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad,
            nft,
            bob_bid_fraction,
            None,
            None
        ));
        let slot = <SlotOf<Test>>::get(nft).unwrap();
        assert_eq!(Ad::slot_current_budget(&slot), bob_bid_fraction);

        let new_budget = 250;
        let new_fungibles = 123;
        assert_noop!(
            Ad::add_budget(
                Origin::signed(BOB),
                ad,
                nft,
                new_budget,
                Some(fungible_id),
                Some(new_fungibles)
            ),
            Error::<Test>::FungibleNotForSlot
        );
    });
}

#[test]
fn should_add_budget() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::force_create(
            Origin::root(),
            9,
            MultiAddress::Id(BOB),
            true,
            1
        ));
        let fungible_id = 9;
        assert_ok!(Assets::mint(
            Origin::signed(BOB),
            fungible_id,
            MultiAddress::Id(BOB),
            1000
        ));

        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        let nft = Nft::preferred(DID_ALICE).unwrap();
        let ad = <Metadata<Test>>::iter_keys().next().unwrap();
        let bob_bid_fraction = 250;
        let bob_bid_fungible = 100;

        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad,
            nft,
            bob_bid_fraction,
            Some(fungible_id),
            Some(bob_bid_fungible)
        ));
        let slot = <SlotOf<Test>>::get(nft).unwrap();
        assert_eq!(Ad::slot_current_budget(&slot), bob_bid_fraction);

        let new_budget = 250;
        let new_fungibles = 123;
        assert_ok!(Ad::add_budget(
            Origin::signed(BOB),
            ad,
            nft,
            new_budget,
            Some(fungible_id),
            Some(new_fungibles)
        ));
        assert_eq!(
            Assets::balance(slot.fungible_id.unwrap(), slot.budget_pot),
            bob_bid_fungible + new_fungibles
        );
        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &BOB),
            BOB_BALANCE - bob_bid_fraction - new_budget
        );

        assert_eq!(
            Ad::slot_current_budget(&slot),
            bob_bid_fraction + new_budget
        );
    });
}

#[test]
fn should_drawback_when_ad_expired() {
    new_test_ext().execute_with(|| {
        // 1. prepare

        let nft = Nft::preferred(DID_ALICE).unwrap();
        let meta = Nft::meta(nft).unwrap();

        // create ad

        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![],
            [0u8; 64].into(),
            1,
            43200 * 2,
            1u128,
            0,
            10u128,
            None
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        // bid

        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad,
            nft,
            400,
            None,
            None
        ));
        assert_eq!(Assets::balance(meta.token_asset_id, BOB), 101);

        // 2. step in

        System::set_block_number(43200);

        Ad::on_initialize(System::block_number());

        // ensure slot, remain

        assert_eq!(<SlotOf<Test>>::get(nft), None);

        // 3. step in
        System::set_block_number(43200 * 2);

        Ad::on_initialize(System::block_number());

        // ensure remain
        assert_eq!(Assets::balance(meta.token_asset_id, BOB), 501);
    });
}
macro_rules! prepare_pay {
    ($a:expr,$b:expr,$c: expr, $d: expr) => {
        _prepare_pay($a, $b, $c, $d)
    };

    () => {
        _prepare_pay(1u128, 0u128, 10u128, 400u128)
    };
}

type HashOf<T> = <<T as frame_system::Config>::Hashing as Hash>::Output;
type NftOf<T> = <T as parami_nft::Config>::AssetId;
fn _prepare_pay(base: u128, min: u128, max: u128, amount: u128) -> (HashOf<Test>, NftOf<Test>) {
    // 1. prepare

    let nft = Nft::preferred(DID_ALICE).unwrap();
    // create ad

    assert_ok!(Ad::create(
        Origin::signed(BOB),
        vec![
            vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
            vec![5u8, 4u8, 3u8, 2u8, 1u8, 0u8]
        ],
        [0u8; 64].into(),
        1,
        10,
        base,
        min,
        max,
        None
    ));

    let ads = <AdsOf<Test>>::get(DID_BOB).unwrap();
    let ad = ads.get(ads.len() - 1).unwrap();

    // bid
    assert_ok!(Ad::bid_with_fraction(
        Origin::signed(BOB),
        ad.clone(),
        nft,
        amount,
        None,
        None
    ));

    return (ad.clone(), nft);
}

#[test]
fn should_pay() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();

        // 2. pay
        assert_ok!(Advertiser::deposit(Origin::signed(BOB), 10 * DOLLARS));
        assert_ok!(Ad::pay(
            Origin::signed(BOB),
            ad,
            nft,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None
        ));

        let nft_meta = Nft::meta(nft).unwrap();
        assert_eq!(Assets::balance(nft_meta.token_asset_id, &CHARLIE), 502);

        // initially charlie'score is 5 + 0 (in + ext), after get a rating of 5, charlie's score is 5 + (0 * 0.8 + 5 * 10 * 0.2) = 15
        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            10
        );
    });
}

#[test]
fn should_pay_3_for_taga5_tagb2() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();
        let nft_meta = Nft::meta(nft).unwrap();
        // 2 pay
        assert_ok!(Ad::pay(
            Origin::signed(BOB),
            ad,
            nft,
            DID_TAGA5_TAGB2,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None
        ));

        assert_eq!(Assets::balance(nft_meta.token_asset_id, &TAGA5_TAGB2), 3);
    });
}

#[test]
fn should_pay_0_when_all_tags_score_are_zero() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();
        let nft_meta = Nft::meta(nft).unwrap();
        // 2 pay
        assert_ok!(Ad::pay(
            Origin::signed(BOB),
            ad,
            nft,
            DID_TAGA0_TAGB0,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None
        ));

        assert_eq!(Assets::balance(nft_meta.token_asset_id, &TAGA0_TAGB0), 0);
    });
}

#[test]
fn should_pay_5_when_all_tags_score_are_zero_with_payout_min_is_5() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!(1u128, 5u128, 10u128, 400u128);
        let nft_meta = Nft::meta(nft).unwrap();
        // 2 pay
        assert_ok!(Ad::pay(
            Origin::signed(BOB),
            ad,
            nft,
            DID_TAGA0_TAGB0,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None
        ));

        assert_eq!(Assets::balance(nft_meta.token_asset_id, &TAGA0_TAGB0), 5);
    });
}
#[test]
fn should_pay_10_when_all_tags_are_full_score() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();
        let nft_meta = Nft::meta(nft).unwrap();
        // 2 pay
        assert_ok!(Ad::pay(
            Origin::signed(BOB),
            ad,
            nft,
            DID_TAGA100_TAGB100,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None
        ));

        assert_eq!(
            Assets::balance(nft_meta.token_asset_id, &TAGA100_TAGB100),
            10
        );
    });
}
#[test]
fn should_pay_10_when_all_tags_are_full_score_or_overflow() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();
        let nft_meta = Nft::meta(nft).unwrap();
        // 2 pay
        assert_ok!(Ad::pay(
            Origin::signed(BOB),
            ad,
            nft,
            DID_TAGA120_TAGB0,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None
        ));

        assert_eq!(Assets::balance(nft_meta.token_asset_id, &TAGA120_TAGB0), 10);
    });
}

#[test]
fn should_pay_dual() {
    new_test_ext().execute_with(|| {
        // 1. prepare

        let nft = Nft::preferred(DID_ALICE).unwrap();

        // create ad

        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        assert_ok!(Assets::force_create(
            Origin::root(),
            9,
            MultiAddress::Id(BOB),
            true,
            1
        ));
        assert_ok!(Assets::mint(
            Origin::signed(BOB),
            9,
            MultiAddress::Id(BOB),
            1000
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        // bid
        assert_eq!(Assets::balance(9, BOB), 1000);
        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad,
            nft,
            400,
            Some(9),
            Some(400)
        ));

        // 2. pay
        assert_ok!(Ad::pay(
            Origin::signed(BOB),
            ad,
            nft,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None
        ));

        let slot = <SlotOf<Test>>::get(nft).unwrap();
        assert_eq!(slot.fungible_id, Some(9));
        assert_eq!(Assets::balance(9, &CHARLIE), 5);
    });
}

#[test]
fn fail_to_pay_if_not_owner_or_delegated() {
    new_test_ext().execute_with(|| {
        // 1. prepare

        let nft = Nft::preferred(DID_ALICE).unwrap();

        // create ad

        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        assert_ok!(Assets::force_create(
            Origin::root(),
            9,
            MultiAddress::Id(BOB),
            true,
            1
        ));
        assert_ok!(Assets::mint(
            Origin::signed(BOB),
            9,
            MultiAddress::Id(BOB),
            1000
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        // bid

        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad,
            nft,
            13,
            Some(9),
            Some(13)
        ));

        // 2. pay
        assert_noop!(
            Ad::pay(
                Origin::signed(ALICE),
                ad,
                nft,
                DID_CHARLIE,
                vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
                None
            ),
            Error::<Test>::NotOwnedOrDelegated
        );
    });
}

#[test]
fn should_pay_if_delegated() {
    new_test_ext().execute_with(|| {
        // 1. prepare

        let nft = Nft::preferred(DID_ALICE).unwrap();

        // create ad

        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            Some(DID_ALICE)
        ));

        assert_ok!(Assets::force_create(
            Origin::root(),
            9,
            MultiAddress::Id(BOB),
            true,
            1
        ));
        assert_ok!(Assets::mint(
            Origin::signed(BOB),
            9,
            MultiAddress::Id(BOB),
            1000
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        // bid

        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad,
            nft,
            13,
            Some(9),
            Some(13)
        ));

        // 2. pay
        assert_ok!(Ad::pay(
            Origin::signed(ALICE),
            ad,
            nft,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None
        ));
    });
}

#[test]
fn should_distribute_fractions_proportionally() {
    use sp_runtime::MultiAddress;

    new_test_ext().execute_with(|| {
        // 1. prepare

        let nft = Nft::preferred(DID_ALICE).unwrap();

        // create ad

        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        assert_ok!(Assets::force_create(
            Origin::root(),
            9,
            MultiAddress::Id(BOB),
            true,
            1
        ));
        assert_ok!(Assets::mint(
            Origin::signed(BOB),
            9,
            MultiAddress::Id(BOB),
            1000
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        // bid
        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad,
            nft,
            10,
            Some(9),
            Some(2)
        ));

        // 2. claim
        let bob_secret_pair: sp_core::sr25519::Pair =
            sp_core::sr25519::Pair::from_string("/Bob", None).unwrap();
        let bod_account_id_32 = AccountId32::new(bob_secret_pair.public().as_array_ref().clone());

        let msg = Ad::construct_claim_sig_msg(
            &ad,
            nft,
            &DID_CHARLIE,
            &vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            &None,
        );
        let signature = bob_secret_pair.sign(msg.as_slice());

        let slot = SlotOf::<Test>::get(nft).unwrap();
        // Charlie's score is 5, and payout base is 1, so the expected received amount is 5 * 1. It's 5 of total 10 fractions.

        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &slot.budget_pot),
            10
        );
        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &CHARLIE),
            500
        );
        assert_eq!(
            Assets::balance(slot.fungible_id.unwrap(), slot.budget_pot),
            2
        );
        assert_eq!(Assets::balance(slot.fungible_id.unwrap(), CHARLIE), 0);
        assert_ok!(Ad::claim(
            Origin::signed(CHARLIE),
            ad,
            nft,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None,
            sp_runtime::MultiSignature::Sr25519(signature),
            bod_account_id_32.clone(),
        ));

        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &slot.budget_pot),
            5
        );
        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &CHARLIE),
            505
        );

        // respectively, we should give 1 fungibles of total 2.
        assert_eq!(
            Assets::balance(slot.fungible_id.unwrap(), slot.budget_pot),
            1
        );
        assert_eq!(Assets::balance(slot.fungible_id.unwrap(), CHARLIE), 1);
    });
}

#[test]
fn should_claim_all_fractions_if_fractions_less_than_expected() {
    use sp_runtime::MultiAddress;

    new_test_ext().execute_with(|| {
        // 1. prepare

        let nft = Nft::preferred(DID_ALICE).unwrap();

        // create ad
        assert_ok!(Advertiser::deposit(Origin::signed(BOB), 10 * DOLLARS));

        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        assert_ok!(Assets::force_create(
            Origin::root(),
            9,
            MultiAddress::Id(BOB),
            true,
            1
        ));
        assert_ok!(Assets::mint(
            Origin::signed(BOB),
            9,
            MultiAddress::Id(BOB),
            1000
        ));

        let ad = <Metadata<Test>>::iter_keys().next().unwrap();

        // bid
        assert_ok!(Ad::bid_with_fraction(
            Origin::signed(BOB),
            ad,
            nft,
            4,
            Some(9),
            Some(13)
        ));

        // 2. claim
        let bob_secret_pair: sp_core::sr25519::Pair =
            sp_core::sr25519::Pair::from_string("/Bob", None).unwrap();
        let bod_account_id_32 = AccountId32::new(bob_secret_pair.public().as_array_ref().clone());

        let msg = Ad::construct_claim_sig_msg(
            &ad,
            nft,
            &DID_CHARLIE,
            &vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            &None,
        );
        let signature = bob_secret_pair.sign(msg.as_slice());

        let slot = SlotOf::<Test>::get(nft).unwrap();
        // Charlie's score is 5, and payout base is 1, so the expected received amount is 5 * 1. However the budget pot has only 4.

        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &slot.budget_pot),
            4
        );
        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &CHARLIE),
            500
        );
        assert_ok!(Ad::claim(
            Origin::signed(CHARLIE),
            ad,
            nft,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None,
            sp_runtime::MultiSignature::Sr25519(signature),
            bod_account_id_32.clone(),
        ));

        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &slot.budget_pot),
            0
        );
        assert_eq!(
            AdAsset::<Test>::reduciable_balance(&slot.ad_asset, &CHARLIE),
            504
        );

        // And our ad is drawback.
        assert_eq!(SlotOf::<Test>::get(nft), None);
    });
}

#[test]
fn should_claim_success_when_signature_exists() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();

        // 2. generate sig
        assert_ok!(Advertiser::deposit(Origin::signed(BOB), 10 * DOLLARS));

        let bob_secret_pair: sp_core::sr25519::Pair =
            sp_core::sr25519::Pair::from_string("/Bob", None).unwrap();
        let bod_account_id_32 = AccountId32::new(bob_secret_pair.public().as_array_ref().clone());
        println!(
            "ss58 address for BOB is {:?}",
            bob_secret_pair.public().as_slice()
        );
        let msg = Ad::construct_claim_sig_msg(
            &ad,
            nft,
            &DID_CHARLIE,
            &vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            &None,
        );
        let signature = bob_secret_pair.sign(msg.as_slice());

        // 3. claim
        let res = Ad::claim(
            Origin::signed(CHARLIE),
            ad,
            nft,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None,
            sp_runtime::MultiSignature::Sr25519(signature),
            bod_account_id_32,
        );

        assert_ok!(res);

        let nft_meta = Nft::meta(nft).unwrap();
        assert_eq!(Assets::balance(nft_meta.token_asset_id, &CHARLIE), 502);

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            10
        );
    });
}

#[test]
fn should_claim_success_when_signature_not_exists() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();

        // 2. claim

        let res = Ad::claim_without_advertiser_signature(
            Origin::signed(CHARLIE),
            ad,
            nft,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None,
        );

        assert_ok!(res);

        let nft_meta = Nft::meta(nft).unwrap();
        assert_eq!(Assets::balance(nft_meta.token_asset_id, &CHARLIE), 502);

        // previous: (intrinsic, extrinsic) = (5, 0)
        // after: (intrinsic, extrinsic) = (5, -5)
        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            0 // Curious, right? It's a ridiculous implementation
        );
    });
}
#[test]
fn should_not_reward_if_score_is_zero() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!(1u128, 1u128, 10u128, 10u128);

        // 2. claim
        assert_ok!(Ad::claim_without_advertiser_signature(
            Origin::signed(CHARLIE),
            ad,
            nft,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None,
        ));

        System::set_block_number(1);
        let (ad, nft) = prepare_pay!(1u128, 1u128, 10u128, 15u128);

        assert_ok!(Ad::claim_without_advertiser_signature(
            Origin::signed(CHARLIE),
            ad,
            nft,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None,
        ));

        let nft_meta = Nft::meta(nft).unwrap();
        let balance = Assets::balance(nft_meta.token_asset_id, &CHARLIE);

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            -5
        );

        System::set_block_number(2);
        let (ad, nft) = prepare_pay!(1u128, 1u128, 10u128, 30u128);

        assert_ok!(Ad::claim_without_advertiser_signature(
            Origin::signed(CHARLIE),
            ad,
            nft,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None,
        ));

        assert_eq!(Assets::balance(nft_meta.token_asset_id, &CHARLIE), balance);
    });
}

#[test]
pub fn non_advertisers_should_not_affect_ratin_when_score_diff_is_positive() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();

        // 2. generate sig
        let bob_secret_pair: sp_core::sr25519::Pair =
            sp_core::sr25519::Pair::from_string("/Bob", None).unwrap();
        let bod_account_id_32 = AccountId32::new(bob_secret_pair.public().as_array_ref().clone());
        println!(
            "ss58 address for BOB is {:?}",
            bob_secret_pair.public().as_slice()
        );
        let msg = Ad::construct_claim_sig_msg(
            &ad,
            nft,
            &DID_CHARLIE,
            &vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            &None,
        );
        let signature = bob_secret_pair.sign(msg.as_slice());

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            5
        );
        // 3. claim
        let res = Ad::claim(
            Origin::signed(CHARLIE),
            ad,
            nft,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)],
            None,
            sp_runtime::MultiSignature::Sr25519(signature),
            bod_account_id_32,
        );

        assert_ok!(res);

        let nft_meta = Nft::meta(nft).unwrap();
        assert_eq!(Assets::balance(nft_meta.token_asset_id, &CHARLIE), 502);

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            5
        );
    });
}

#[test]
pub fn non_advertisers_should_not_affect_ratin_when_score_diff_is_negative() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let (ad, nft) = prepare_pay!();

        // 2. generate sig
        let bob_secret_pair: sp_core::sr25519::Pair =
            sp_core::sr25519::Pair::from_string("/Bob", None).unwrap();
        let bod_account_id_32 = AccountId32::new(bob_secret_pair.public().as_array_ref().clone());
        println!(
            "ss58 address for BOB is {:?}",
            bob_secret_pair.public().as_slice()
        );
        let msg = Ad::construct_claim_sig_msg(
            &ad,
            nft,
            &DID_CHARLIE,
            &vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], -5)],
            &None,
        );
        let signature = bob_secret_pair.sign(msg.as_slice());

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            5
        );
        // 3. claim
        let res = Ad::claim(
            Origin::signed(CHARLIE),
            ad,
            nft,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], -5)],
            None,
            sp_runtime::MultiSignature::Sr25519(signature),
            bod_account_id_32,
        );

        assert_ok!(res);

        let nft_meta = Nft::meta(nft).unwrap();
        assert_eq!(Assets::balance(nft_meta.token_asset_id, &CHARLIE), 502);

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            0
        );
    });
}

#[test]
fn should_rate_after_claim_without_score() {
    new_test_ext().execute_with(|| {
        let (ad, nft) = prepare_pay!();

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            5
        );

        let res = Ad::claim_without_advertiser_signature(
            Origin::signed(CHARLIE),
            ad,
            nft,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], -5)],
            None,
        );
        assert_ok!(res);

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            0
        );

        assert_ok!(Ad::rate(
            Origin::signed(BOB),
            ad,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)]
        ));

        assert_eq!(
            Tag::get_score(&DID_CHARLIE, vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8]),
            10
        );
    });
}

#[test]
fn should_fail_if_rated_again() {
    new_test_ext().execute_with(|| {
        let (ad, nft) = prepare_pay!();

        let res = Ad::claim_without_advertiser_signature(
            Origin::signed(CHARLIE),
            ad,
            nft,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], -5)],
            None,
        );
        assert_ok!(res);

        assert_ok!(Ad::rate(
            Origin::signed(BOB),
            ad,
            DID_CHARLIE,
            vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)]
        ));

        assert_noop!(
            Ad::rate(
                Origin::signed(BOB),
                ad,
                DID_CHARLIE,
                vec![(vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], 5)]
            ),
            Error::<Test>::Rated
        );
    });
}

#[test]
fn should_fail_if_payout_base_too_low() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ad::create(
                Origin::signed(BOB),
                vec![],
                [0u8; 64].into(),
                1,
                43200 * 2,
                0u128,
                0,
                10u128,
                None
            ),
            Error::<Test>::PayoutBaseTooLow
        );

        assert_noop!(
            Ad::create(
                Origin::signed(BOB),
                vec![],
                [0u8; 64].into(),
                1,
                43200 * 2,
                10u128,
                0,
                5u128,
                None
            ),
            Error::<Test>::WrongPayoutSetting
        );
    });
}

#[test]
fn should_bid_with_currency() {
    new_test_ext().execute_with(|| {
        // 1. prepare

        let nft = 0;
        let meta = Nft::meta(nft).unwrap();
        let endtime = 43200;
        let charlie_balance = Balances::free_balance(&CHARLIE);

        // ad1
        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![],
            [0u8; 64].into(),
            1,
            endtime,
            1u128,
            0,
            10u128,
            None
        ));

        let ad1 = <Metadata<Test>>::iter_keys().next().unwrap();

        // 2. bob bid for ad1

        let slot = <SlotOf<Test>>::get(nft);
        assert_eq!(slot, None);

        let bob_bid_currency = 400;

        assert_ok!(Ad::bid_with_currency(
            Origin::signed(BOB),
            ad1,
            nft,
            bob_bid_currency,
        ));

        // ensure: deadline, slot, remain
        assert_eq!(<EndtimeOf<Test>>::get(&ad1), Some(endtime));
        assert_eq!(<DeadlineOf<Test>>::get(nft, &ad1), Some(endtime));

        let slot = <SlotOf<Test>>::get(nft).unwrap();
        assert_eq!(slot.ad_id, ad1);
        let ad_asset = &slot.ad_asset;
        assert_eq!(*ad_asset, CurrencyOrAsset::<u32>::Currency);

        // 3. charlie bid for ad2
        // ad2

        assert_ok!(Ad::create(
            Origin::signed(CHARLIE),
            vec![],
            [0u8; 64].into(),
            1,
            1,
            1u128,
            0,
            10u128,
            None
        ));

        let ad2 = <Metadata<Test>>::iter_keys().next().unwrap();

        assert_noop!(
            Ad::bid_with_currency(
                Origin::signed(CHARLIE),
                ad2,
                nft,
                bob_bid_currency.saturating_mul(120).saturating_div(100),
            ),
            Error::<Test>::Underbid
        );

        assert_eq!(
            AdAsset::<Test>::reduciable_balance(ad_asset, &CHARLIE),
            charlie_balance
        );
        let charlie_bid_currency = bob_bid_currency
            .saturating_mul(120)
            .saturating_div(100)
            .saturating_add(1);

        assert_ok!(Ad::bid_with_currency(
            Origin::signed(CHARLIE),
            ad2,
            nft,
            charlie_bid_currency,
        ));
        assert_eq!(
            AdAsset::<Test>::reduciable_balance(ad_asset, &CHARLIE),
            charlie_balance - charlie_bid_currency
        );

        let slot = <SlotOf<Test>>::get(nft).unwrap();
        assert_eq!(slot.ad_id, ad2);

        let locked_budget = AdAsset::<Test>::reduciable_balance(ad_asset, &slot.budget_pot);
        assert_eq!(locked_budget, charlie_bid_currency);

        // ensure: deadline, slot, remain

        assert_eq!(<EndtimeOf<Test>>::get(&ad2), Some(1));
        assert_eq!(<DeadlineOf<Test>>::get(nft, &ad1), None);
        assert_eq!(<DeadlineOf<Test>>::get(nft, &ad2), Some(1));
    });
}

#[test]
fn should_fail_bid_with_currency_if_minted() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let nft = 1;
        let endtime = 43200;

        // ad1
        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![],
            [0u8; 64].into(),
            1,
            endtime,
            1u128,
            0,
            10u128,
            None
        ));

        let ad1 = <Metadata<Test>>::iter_keys().next().unwrap();

        // 2. bob bid for ad1

        let slot = <SlotOf<Test>>::get(nft);
        assert_eq!(slot, None);

        let bob_bid_currency = 400;

        assert_noop!(
            Ad::bid_with_currency(Origin::signed(CHARLIE), ad1, nft, bob_bid_currency),
            Error::<Test>::Minted
        );
    });
}

#[test]
fn should_fail_bid_with_fraction_if_not_minted() {
    new_test_ext().execute_with(|| {
        // 1. prepare
        let nft = 0;
        let endtime = 43200;

        // ad1
        assert_ok!(Ad::create(
            Origin::signed(BOB),
            vec![],
            [0u8; 64].into(),
            1,
            endtime,
            1u128,
            0,
            10u128,
            None
        ));

        let ad1 = <Metadata<Test>>::iter_keys().next().unwrap();

        // 2. bob bid for ad1

        let slot = <SlotOf<Test>>::get(nft);
        assert_eq!(slot, None);

        let bob_bid_fraction = 400;

        assert_noop!(
            Ad::bid_with_fraction(Origin::signed(BOB), ad1, nft, bob_bid_fraction, None, None),
            Error::<Test>::NotMinted
        );
    });
}
