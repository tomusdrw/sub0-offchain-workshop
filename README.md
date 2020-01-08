### 1. Clone repo:
```bash
$ git clone --depth 1 --branch td-submit-transactions https://github.com/paritytech/substrate
```

### 2. Add offchain worker that prints `Hello World!'
Compile & run
```bash
$ cargo run -p node-template -- --dev -lruntime=trace
```

### 3. Either print or `debug::warn`, also show logger
```rust
 debug::RuntimeLogger::init();
 debug::print!("Hello world!");
```

### 4. Read `Something` from the storage and display block hash/number

```rust
fn offchain_worker(block_number: T::BlockNumber) {
	//debug::RuntimeLogger::init();
	let something = Something::get();
	debug::warn!("Hello World from offchain workers!");
	debug::warn!("Something is: {:?}", something);

	let block_hash = <system::Module<T>>::block_hash(block_number - 1);
	debug::warn!("Current block is: {:?} (parent: {:?})", block_number, block_hash);
}
```


### 5. Make an HTTP request.

#### 5.1. Handle response.

```rust
let price = match Self::fetch_btc_price() {
    Ok(price) => {
      debug::info!("Got BTC price: {} cents", price);
      price
    },
    _ => {
      debug::error!("Error fetching BTC price.", e);
      return
    }
};
```

#### 5.2. Make a request and check response.

```rust
  impl<T: Trait> Module<T> {
    fn fetch_btc_price() -> Result<u64, http::Error> {
      let pending = http::Request::get(
        "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD"
      ).send().map_err(|_| http::Error::IoError)?;

      let response = pending.wait()?;
      if response.code != 200 {
        debug::warn!("Unexpected status code: {}", response.code);
        return Err(http::Error::Unknown);
      }

      let body = response.body().collect::<Vec<u8>>();
      debug::warn!("Body: {:?}", core::str::from_utf8(&body).ok());

      Ok(5)
    }
  }
```

#### 5.3. Parse JSON.

```rust
	const START_IDX: usize = "{\"USD\":".len();
	let body = response.body().collect::<Vec<u8>>();
	let json = match core::str::from_utf8(&body) {
		Ok(json) if json.len() > START_IDX => json,
		_ => {
			debug::warn!("Unexpected (non-utf8 or too short) response received: {:?}", body);
			return Err(http::Error::Unknown);
		}
	};
```

#### 5.4. Parse number:

```rust
	let price = &json[START_IDX .. json.len() - 1];
	let pricef: f64 = match price.parse() {
		Ok(pricef) => pricef,
		Err(_) => {
			debug::warn!("Unparsable price: {:?}", price);
			return Err(http::Error::Unknown);
		}
	};

	Ok((pricef * 100.) as u64)
```

### 6. Create signed transaction.

#### 6.1. Add type SubmitTransaction, type Call to the trait.

```rust
	/// The overarching event type.
	type Call: From<Call<Self>>;

	/// Transaction submitter.
	type SubmitTransaction: system::offchain::SubmitSignedTransaction<Self, <Self as Trait>::Call>;
```

#### 6.2. Add app-crypto stuff.

```rust
  use primitives::crypto::KeyTypeId;
  pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");

  pub mod crypto {
    use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, KEY_TYPE);
  }
```

#### 6.3. Implement CreateTransaction for the runtime.

```rust
  /// The payload being signed in transactions.
  pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

  impl system::offchain::CreateTransaction<Runtime, UncheckedExtrinsic> for Runtime {
    type Public = <Signature as sp_runtime::traits::Verify>::Signer;
    type Signature = Signature;

    fn create_transaction<TSigner: system::offchain::Signer<Self::Public, Self::Signature>>(
      call: Call,
      public: Self::Public,
      account: AccountId,
      index: Index,
    ) -> Option<(Call, <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> {
      let period = 1 << 8;
      let current_block = System::block_number() as u64;
      let tip = 0;
      let extra: SignedExtra = (
        system::CheckVersion::<Runtime>::new(),
        system::CheckGenesis::<Runtime>::new(),
        system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
        system::CheckNonce::<Runtime>::from(index),
        system::CheckWeight::<Runtime>::new(),
        transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
      );
      let raw_payload = SignedPayload::new(call, extra).ok()?;
      let signature = TSigner::sign(public, &raw_payload)?;
      let address = Indices::unlookup(account);
      let (call, extra, _) = raw_payload.deconstruct();
      Some((call, (address, signature, extra)))
    }
  }
```

#### 6.4. Implement submission logic.

```rust
fn submit_btc_price_on_chain(price: u32) {
	use system::offchain::SubmitSignedTransaction;

	let call = Call::do_something(price);

	let res = T::SubmitTransaction::submit_signed(
		call
	);

	if res.is_empty() {
		debug::error!("No local accounts found.");
	}
}
```

### 7. Testing

#### 7.1. Generate a new account

```bash
$ subkey -s generate
```

#### 7.2. Submit  a new key via RPC

```bash
$ http localhost:9933 jsonrpc=2.0 id=1 method=author_insertKey params:='["btc!", "garment disorder company wasp craft dinosaur street crucial salad door maid document", "0xc44c1627a435c00e40bced87e2361236ced5b8db8aa6c1dd248926fe743f832f"]'
```

#### 7.3. Transfer the balance

In the UI https://polkadot.js.org/apps/#/explorer

#### 7.4. Alternatively insert account (Alice) to the keystore during CLI init.

https://github.com/gnunicorn/substrate-offchain-cb/blob/df0dbca/src/service.rs#L116

### 8. Implement handling the incoming prices.

```rust
	pub fn submit_btc_price(origin, price: u32) -> dispatch::Result {
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

		let block_hash = <system::Module<T>>::block_hash(block_number);
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
```


##### TODO substrate
1. Move sr_primitives::offchain to offchain-primitives
2. Re-export types from `primitives/core/offchain/mod.rs` (HttpError)?
3. Inconsistency in error types (HttpError vs Error), make the former internal?
5. Annoying WASM warnings.

4. Make vector of accounts optional - use all accounts.
6. Add must_use to submit_signed return value.
