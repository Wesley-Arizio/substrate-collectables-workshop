use super::*;
use frame::prelude::*;
use frame::traits::BlakeTwo256;
use frame::traits::Hash;

impl<T: Config> Pallet<T> {
	pub fn gen_dna() -> [u8; 32] {
		let parent_hash = frame_system::Pallet::<T>::parent_hash();
		let block_number = frame_system::Pallet::<T>::block_number();
		let extrinsic_index = frame_system::Pallet::<T>::extrinsic_index();
		let kitties_count = CountForKitties::<T>::get();

		let input = (parent_hash, block_number, extrinsic_index, kitties_count);
		BlakeTwo256::hash_of(&input).into()
	}

	pub fn mint(owner: T::AccountId, dna: [u8; 32]) -> DispatchResult {
		ensure!(!Kitties::<T>::contains_key(dna), Error::<T>::DuplicateKitty);
		let current_count: u32 = CountForKitties::<T>::get();
		let new_count = current_count.checked_add(1).ok_or(Error::<T>::TooManyKitties)?;

		let kitty = Kitty { dna, owner: owner.clone(), price: None };

		KittiesOwned::<T>::try_append(&owner, dna).map_err(|_| Error::<T>::TooManyOwned)?;
		Kitties::<T>::insert(dna, kitty);
		CountForKitties::<T>::set(new_count);
		Self::deposit_event(Event::<T>::Created { owner });
		Ok(())
	}

	pub fn do_transfer(from: T::AccountId, to: T::AccountId, kitty_id: [u8; 32]) -> DispatchResult {
		ensure!(from != to, Error::<T>::TransferToSelf);

		let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;
		ensure!(kitty.owner == from, Error::<T>::NotOwner);

		kitty.owner = to.clone();
		kitty.price = None;

		let mut receiver = KittiesOwned::<T>::get(&to);
		receiver.try_push(kitty_id).map_err(|_| Error::<T>::TooManyOwned)?;

		let mut sender = KittiesOwned::<T>::get(&from);
		if let Some(ind) = sender.iter().position(|&id| id == kitty_id) {
			sender.swap_remove(ind);
		} else {
			return Err(Error::<T>::NoKitty.into())
		}

		Kitties::<T>::insert(kitty_id, kitty);
		KittiesOwned::<T>::insert(&from, sender);
		KittiesOwned::<T>::insert(&to, receiver);

		Self::deposit_event(Event::<T>::Transferred { from, to, kitty_id });
		Ok(())
	}
}
