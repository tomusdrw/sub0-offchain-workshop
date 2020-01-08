// Copyright 2018-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use log::warn;

pub fn internal<E: ::std::fmt::Debug>(e: E) -> jsonrpc_core::Error {
	warn!("Unknown error: {:?}", e);
	jsonrpc_core::Error {
		code: jsonrpc_core::ErrorCode::InternalError,
		message: "Unknown error occured".into(),
		data: Some(format!("{:?}", e).into()),
	}
}
