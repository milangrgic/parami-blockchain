use crate as parami_ad;
use frame_support::traits::{ConstU128, ConstU32};
use frame_support::{
    parameter_types, traits::AsEnsureOriginWithArg, traits::GenesisBuild, PalletId,
};
use frame_system::{self as system, EnsureRoot, EnsureSigned};
use parami_primitives::constants::DOLLARS;
use sp_core::{sr25519, H160, H256};
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Keccak256},
};

type UncheckedExtrinsic = system::mocking::MockUncheckedExtrinsic<Test>;
type Block = system::mocking::MockBlock<Test>;

pub type Extrinsic = TestXt<Call, ()>;

pub const ALICE: sr25519::Public = sr25519::Public([1; 32]);
//public key of secret phrase "/Bob"
pub const BOB: sr25519::Public = sr25519::Public([
    202, 53, 5, 56, 6, 22, 158, 222, 140, 155, 250, 77, 126, 205, 109, 249, 77, 192, 15, 146, 38,
    242, 60, 222, 145, 142, 98, 183, 108, 10, 190, 123,
]);
pub const CHARLIE: sr25519::Public = sr25519::Public([3; 32]);
pub const TAGA5_TAGB2: sr25519::Public = sr25519::Public([4; 32]);
pub const TAGA0_TAGB0: sr25519::Public = sr25519::Public([5; 32]);
pub const TAGA100_TAGB100: sr25519::Public = sr25519::Public([8; 32]);
pub const TAGA120_TAGB0: sr25519::Public = sr25519::Public([9; 32]);

pub const BOB_BALANCE: u128 = 501;
pub const CHARLIE_BALANCE: u128 = 500;

pub const DID_ALICE: H160 = H160([0xff; 20]);
pub const DID_BOB: H160 = H160([0xee; 20]);
pub const DID_CHARLIE: H160 = H160([0xdd; 20]);
pub const DID_TAGA5_TAGB2: H160 = H160([0x1; 20]);
pub const DID_TAGA0_TAGB0: H160 = H160([0x2; 20]);
pub const DID_TAGA100_TAGB100: H160 = H160([0x3; 20]);
pub const DID_TAGA120_TAGB0: H160 = H160([0x4; 20]);
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: system::{Pallet, Call, Config, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Uniques: pallet_uniques::{Pallet, Storage, Event<T>},

        Did: parami_did::{Pallet, Call, Storage, Config<T>, Event<T>},
        Nft: parami_nft::{Pallet, Call, Storage, Event<T>},
        Ocw: parami_ocw::{Pallet},
        Swap: parami_swap::{Pallet, Call, Storage, Event<T>},
        Tag: parami_tag::{Pallet, Call, Storage, Config<T>, Event<T>},
        Ad: parami_ad::{Pallet, Call, Storage, Event<T>},
        AssetIdManager: parami_assetmanager::{Pallet},
        Advertiser: parami_advertiser::{Pallet, Call, Storage, Event<T>}
    }
);

type AssetId = u32;
type Balance = u128;
type BlockNumber = u64;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type MaxConsumers = ConstU32<16>;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sr25519::Public;
    type Lookup = Did;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

parameter_types! {
    pub const AssetDeposit: Balance = 100;
    pub const ApprovalDeposit: Balance = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Test {
    type AssetAccountDeposit = ConstU128<10000u128>;
    type Event = Event;
    type Balance = Balance;
    type AssetId = AssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const ClassDeposit: Balance = 0;
    pub const InstanceDeposit: Balance = 0;
    pub const AttributeDepositBase: Balance = 0;
}

impl pallet_uniques::Config for Test {
    type Event = Event;
    type CollectionId = AssetId;
    type ItemId = AssetId;
    type Currency = Balances;
    type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type CollectionDeposit = ClassDeposit;
    type ItemDeposit = InstanceDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type AttributeDepositBase = AttributeDepositBase;
    type DepositPerByte = MetadataDepositPerByte;
    type StringLimit = StringLimit;
    type KeyLimit = StringLimit;
    type ValueLimit = StringLimit;
    type WeightInfo = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
    type Locker = ();
}

impl parami_did::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type DecentralizedId = H160;
    type Hashing = Keccak256;
    type WeightInfo = ();
    type Transferables = (Nft,);
}

parameter_types! {
    pub const DefaultInitialMintingDeposit: Balance = 1_000;
    pub const InitialMintingLockupPeriod: BlockNumber = 5;
    pub const InitialMintingValueBase: Balance = 1_000_000;
    pub const PendingLifetime: BlockNumber = 5;
    pub const NftPalletId: PalletId = PalletId(*b"prm/nft ");
}

impl parami_assetmanager::Config for Test {
    type AssetId = u32;
}

impl parami_nft::Config for Test {
    type Event = Event;
    type AssetId = AssetId;
    type Assets = Assets;
    type DefaultInitialMintingDeposit = DefaultInitialMintingDeposit;
    type InitialMintingLockupPeriod = InitialMintingLockupPeriod;
    type InitialMintingValueBase = InitialMintingValueBase;
    type Links = ();
    type Nft = Uniques;
    type PalletId = NftPalletId;
    type PendingLifetime = PendingLifetime;
    type StringLimit = StringLimit;
    type Swaps = Swap;
    type WeightInfo = ();
    type UnsignedPriority = ();
    type AssetIdManager = AssetIdManager;
    type NftId = u32;
}

impl parami_ocw::Config for Test {}

parameter_types! {
    pub const SwapPalletId: PalletId = PalletId(*b"prm/swap");
    pub const StakingRewardAmountParam: u128 = 7_000_000 * DOLLARS;
}

impl parami_swap::Config for Test {
    type Event = Event;
    type AssetId = AssetId;
    type Assets = Assets;
    type Currency = Balances;
    type PalletId = SwapPalletId;
    type WeightInfo = ();
    type StakingRewardAmount = StakingRewardAmountParam;
    type Stakes = ();
}

parameter_types! {
    pub const SubmissionFee: Balance = 1;
}

impl parami_tag::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type DecentralizedId = H160;
    type SubmissionFee = SubmissionFee;
    type CallOrigin = parami_did::EnsureDid<Self>;
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = ();
}

parameter_types! {
    pub const AdPalletId: PalletId = PalletId(*b"prm/ad  ");
    pub const AdvertiserMinimumFee: Balance = 1;
    pub const AdMinimumPayoutBase: Balance = 1;
    pub const SlotLifetime: BlockNumber = 43200;
    pub const MinimumDeposit: Balance = 10_000_000_000_000_000_000;
    pub const AdvertiserPalletId: PalletId = PalletId(*b"prm/adve");
}

impl parami_ad::Config for Test {
    type Event = Event;
    type MinimumFeeBalance = AdvertiserMinimumFee;
    type MinimumPayoutBase = AdMinimumPayoutBase;
    type PalletId = AdPalletId;
    type SlotLifetime = SlotLifetime;
    type Tags = Tag;
    type CallOrigin = parami_did::EnsureDid<Self>;
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = ();
}

impl parami_advertiser::Config for Test {
    type Event = Event;
    type MinimumDeposit = MinimumDeposit;
    type PalletId = AdvertiserPalletId;
    type Slash = ();
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 100),
            (BOB, 3_000_000_000_000_000_000_000),
            (CHARLIE, 3_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    parami_did::GenesisConfig::<Test> {
        ids: vec![
            (ALICE, DID_ALICE),
            (BOB, DID_BOB),
            (CHARLIE, DID_CHARLIE),
            (TAGA5_TAGB2, DID_TAGA5_TAGB2),
            (TAGA0_TAGB0, DID_TAGA0_TAGB0),
            (TAGA100_TAGB100, DID_TAGA100_TAGB100),
            (TAGA120_TAGB0, DID_TAGA120_TAGB0),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    parami_tag::GenesisConfig::<Test> {
        tag: vec![
            vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8], //Tag: T
            vec![5u8, 4u8, 3u8, 2u8, 1u8, 0u8], //Tag: E
        ],
        personas: vec![
            (
                DID_CHARLIE,
                vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
                parami_tag::types::Score::new(5),
            ),
            (
                DID_TAGA5_TAGB2,
                vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
                parami_tag::types::Score::new(5),
            ),
            (
                DID_TAGA5_TAGB2,
                vec![5u8, 4u8, 3u8, 2u8, 1u8, 0u8],
                parami_tag::types::Score::new(2),
            ),
            (
                DID_TAGA100_TAGB100,
                vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
                parami_tag::types::Score::new(50),
            ),
            (
                DID_TAGA100_TAGB100,
                vec![5u8, 4u8, 3u8, 2u8, 1u8, 0u8],
                parami_tag::types::Score::new(50),
            ),
            (
                DID_TAGA120_TAGB0,
                vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8],
                parami_tag::types::Score::new(40),
            ),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    parami_nft::GenesisConfig::<Test> {
        deposit: vec![],
        deposits: vec![],
        next_instance_id: 2,
        nfts: vec![(0, DID_ALICE, false), (1, DID_ALICE, true)],
        externals: Default::default(),
        validate_endpoints: Default::default(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_assets::GenesisConfig::<Test> {
        assets: vec![(1, ALICE, true, 1)],
        accounts: vec![
            (1, ALICE, 10000),
            (1, BOB, BOB_BALANCE),
            (1, CHARLIE, CHARLIE_BALANCE),
        ],
        metadata: vec![],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
