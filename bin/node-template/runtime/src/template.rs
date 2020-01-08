/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{decl_module, decl_storage, decl_event, dispatch, debug};
use system::{ensure_signed, offchain};
use sp_runtime::offchain::http;
use sp_std::vec::Vec;

use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");

pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, KEY_TYPE);
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// The overarching event type.
	type Call: From<Call<Self>>;

	/// Transaction submitter.
	type SubmitTransaction: offchain::SubmitSignedTransaction<Self, <Self as Trait>::Call>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		Prices get(fn prices): Vec<u32>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		pub fn submit_btc_price(origin, price: u32) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			debug::info!("Adding to the average: {}", price);
			let average = Prices::mutate(|prices| {
				const MAX_LEN: usize = 64;

				if prices.len() < MAX_LEN {
					prices.push(price);
				} else {
					prices[price as usize % MAX_LEN] = price;
				}

				// TODO Whatchout for overflows
				prices.iter().sum::<u32>() / prices.len() as u32
			});
			debug::info!("Current average price is: {}", average);
			// here we are raising the Something event
			Self::deposit_event(RawEvent::NewPrice(price, who));
			Ok(())
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::RuntimeLogger::init();
			let average: Option<u32> = {
				let prices = Prices::get();
				if prices.is_empty() {
					None
				} else {
					Some(prices.iter().sum::<u32>() / prices.len() as u32)
				}
			};
			debug::warn!("Hello World from offchain workers!");
			debug::warn!("Current price of BTC is: {:?}", average);

			let block_hash = <system::Module<T>>::block_hash(block_number - 1.into());
			debug::warn!("Current block is: {:?} ({:?})", block_number, block_hash);

			let price = match Self::fetch_btc_price() {
				Ok(price) => {
					debug::warn!("Got BTC price: {} cents", price);
					price
				},
				Err(_) => {
					debug::warn!("Error fetching BTC price.");
					// TODO [ToDr] What to do here?
					return
				}
			};

			Self::submit_btc_price_on_chain(price);
		}
	}
}

impl<T: Trait> Module<T> {
	fn fetch_btc_price() -> Result<u32, http::Error> {
		let pending = http::Request::get(
			"https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD"
		).send().map_err(|_| http::Error::IoError)?;

		let response = pending.wait()?;
		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown);
		}

		const START_IDX: usize = "{\"USD\":".len();
		let body = response.body().collect::<Vec<u8>>();
		let json = match core::str::from_utf8(&body) {
			Ok(json) if json.len() > START_IDX => json,
			_ => {
				debug::warn!("Unexpected (non-utf8 or too short) response received: {:?}", body);
				return Err(http::Error::Unknown);
			}
		};

		let price = &json[START_IDX .. json.len() - 1];
		let pricef: f64 = match price.parse() {
			Ok(pricef) => pricef,
			Err(_) => {
				debug::warn!("Unparsable price: {:?}", price);
				return Err(http::Error::Unknown);
			}
		};

		Ok((pricef * 100.) as u32)
	}

	fn submit_btc_price_on_chain(price: u32) {
		use system::offchain::SubmitSignedTransaction;

		let call = Call::submit_btc_price(price);
		let res = T::SubmitTransaction::submit_signed(call);

		if res.is_empty() {
			debug::error!("No local accounts found.");
		} else {
			debug::info!("Sent transactions from: {:?}", res);
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		NewPrice(u32, AccountId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::H256;
	use support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		new_test_ext().execute_with(|| {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}
}
