use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU128, ConstU32, Everything, Nothing, ProcessMessage, ProcessMessageError},
	weights::{IdentityFee, WeightMeter},
};
use frame_system::EnsureRoot;
use sp_runtime::{traits::IdentityLookup, AccountId32};

use crate::Weight;
use cumulus_primitives_core::ParaId;
use polkadot_runtime_parachains::{
	configuration,
	inclusion::{AggregateMessageOrigin, UmpQueueId},
	origin, shared,
};
use xcm::v5::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowTopLevelPaidExecutionFrom, ChildParachainAsNative, ChildParachainConvertsVia,
	FixedWeightBounds, FungibleAdapter, IsConcrete, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
};
use xcm_executor::{Config, XcmExecutor};

pub type AccountId = AccountId32;
pub type Balance = u128;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = [u8; 8];
	type MaxFreezes = ();
	type DoneSlashHandler = ();
}

impl shared::Config for Runtime {
	type DisabledValidators = ();
}

impl configuration::Config for Runtime {
	type WeightInfo = configuration::TestWeightInfo;
}

parameter_types! {
	pub KsmLocation: Location = Here.into();
	pub const KusamaNetwork: NetworkId = NetworkId::Kusama;
	pub UniversalLocation: InteriorLocation = [GlobalConsensus(KusamaNetwork::get())].into();
}

pub type SovereignAccountOf = (
	ChildParachainConvertsVia<ParaId, AccountId>,
	AccountId32Aliases<KusamaNetwork, AccountId>,
);

pub type LocalAssetTransactor = FungibleAdapter<Balances, IsConcrete<KsmLocation>, SovereignAccountOf, AccountId, ()>;

type LocalOriginConverter = (
	SovereignSignedViaLocation<SovereignAccountOf, RuntimeOrigin>,
	ChildParachainAsNative<origin::Origin, RuntimeOrigin>,
	SignedAccountId32AsNative<KusamaNetwork, RuntimeOrigin>,
);

pub type XcmRouter = super::RelayChainXcmRouter;
pub type Barrier = (TakeWeightCredit, AllowTopLevelPaidExecutionFrom<Everything>);

parameter_types! {
	pub Kusama: AssetFilter = Wild(AllOf { fun: WildFungible, id: AssetId(KsmLocation::get()) });
	pub Statemine: Location = Parachain(3).into();
	pub KusamaForStatemine: (AssetFilter, Location) = (Kusama::get(), Statemine::get());
}

pub type TrustedTeleporters = xcm_builder::Case<KusamaForStatemine>;

parameter_types! {
	pub const UnitWeightCost: Weight = Weight::from_parts(10, 10);
	pub const BaseXcmWeight: Weight = Weight::from_parts(100_000_000, 100_000_000);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = LocalOriginConverter;
	type IsReserve = ();
	type IsTeleporter = TrustedTeleporters;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = UsingComponents<IdentityFee<Balance>, KsmLocation, AccountId, Balances, ()>;
	type ResponseHandler = ();
	type AssetTrap = ();
	type AssetClaims = ();
	type SubscriptionService = XcmPallet;
	type AssetLocker = XcmPallet;
	type AssetExchanger = ();
	type PalletInstancesInfo = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = ();
	type TransactionalProcessor = ();
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = ();
	type XcmEventEmitter = ();
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, KusamaNetwork>;

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	// Anyone can execute XCM messages locally...
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type AuthorizedAliasConsideration = ();
}

impl origin::Config for Runtime {}

type Block = frame_system::mocking::MockBlock<Runtime>;

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Weight::from_parts(1_000_000_000, 1_000_000);
	pub const MessageQueueHeapSize: u32 = 65_536;
	pub const MessageQueueMaxStale: u32 = 16;
}

pub struct MessageProcessor;
impl ProcessMessage for MessageProcessor {
	type Origin = AggregateMessageOrigin;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
		id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		let para = match origin {
			AggregateMessageOrigin::Ump(UmpQueueId::Para(para)) => para,
		};
		xcm_builder::ProcessXcmMessage::<Junction, xcm_executor::XcmExecutor<XcmConfig>, RuntimeCall>::process_message(
			message,
			Junction::Parachain(para.into()),
			meter,
			id,
		)
	}
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Size = u32;
	type HeapSize = MessageQueueHeapSize;
	type MaxStale = MessageQueueMaxStale;
	type ServiceWeight = MessageQueueServiceWeight;
	type MessageProcessor = MessageProcessor;
	type QueueChangeHandler = ();
	type QueuePausedQuery = ();
	type WeightInfo = ();
	type IdleMaxServiceWeight = ();
}

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		ParasOrigin: origin,
		MessageQueue: pallet_message_queue,
		XcmPallet: pallet_xcm,
	}
);
