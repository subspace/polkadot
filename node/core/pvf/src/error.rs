// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use parity_scale_codec::{Decode, Encode};

/// Result of PVF preparation performed by the validation host.
pub type PrepareResult = Result<(), PrepareError>;

/// An error that occurred during the prepare part of the PVF pipeline.
#[derive(Debug, Clone, Encode, Decode)]
pub enum PrepareError {
	/// During the prevalidation stage of preparation an issue was found with the PVF.
	Prevalidation(String),
	/// Compilation failed for the given PVF.
	Preparation(String),
	/// Failed to prepare the PVF due to the time limit.
	TimedOut,
	/// This state indicates that the process assigned to prepare the artifact wasn't responsible
	/// or were killed. This state is reported by the validation host (not by the worker).
	DidNotMakeIt,
}

/// A error raised during validation of the candidate.
#[derive(Debug, Clone)]
pub enum ValidationError {
	/// The error was raised because the candidate is invalid.
	InvalidCandidate(InvalidCandidate),
	/// This error is raised due to inability to serve the request.
	InternalError(String),
}

/// A description of an error raised during executing a PVF and can be attributed to the combination
/// of the candidate [`polkadot_parachain::primitives::ValidationParams`] and the PVF.
#[derive(Debug, Clone)]
pub enum InvalidCandidate {
	/// The failure is reported by the worker. The string contains the error message.
	///
	/// This also includes the errors reported by the preparation pipeline.
	WorkerReportedError(String),
	/// The worker has died during validation of a candidate. That may fall in one of the following
	/// categories, which we cannot distinguish programmatically:
	///
	/// (a) Some sort of transient glitch caused the worker process to abort. An example would be that
	///     the host machine ran out of free memory and the OOM killer started killing the processes,
	///     and in order to save the parent it will "sacrifice child" first.
	///
	/// (b) The candidate triggered a code path that has lead to the process death. For example,
	///     the PVF found a way to consume unbounded amount of resources and then it either exceeded
	///     an `rlimit` (if set) or, again, invited OOM killer. Another possibility is a bug in
	///     wasmtime allowed the PVF to gain control over the execution worker.
	///
	/// We attribute such an event to an invalid candidate in either case.
	///
	/// The rationale for this is that a glitch may lead to unfair rejecting candidate by a single
	/// validator. If the glitch is somewhat more persistent the validator will reject all candidate
	/// thrown at it and hopefully the operator notices it by decreased reward performance of the
	/// validator. On the other hand, if the worker died because of (b) we would have better chances
	/// to stop the attack.
	AmbigiousWorkerDeath,
	/// PVF execution (compilation is not included) took more time than was allotted.
	HardTimeout,
}

impl From<PrepareError> for ValidationError {
	fn from(error: PrepareError) -> Self {
		let error_str = match error {
			PrepareError::Prevalidation(err) => err,
			PrepareError::Preparation(err) => err,
			PrepareError::TimedOut => "preparation timeout".to_owned(),
			PrepareError::DidNotMakeIt => "communication error".to_owned(),
		};
		ValidationError::InvalidCandidate(InvalidCandidate::WorkerReportedError(error_str))
	}
}
