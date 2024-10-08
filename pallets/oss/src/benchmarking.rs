use crate::{Pallet as OssPallet, *};
use frame_benchmarking::{
	account, benchmarks, impl_benchmark_test_suite, whitelist_account, whitelisted_caller,
};
use frame_support::dispatch::RawOrigin;
use sp_runtime::{
	AccountId32, MultiSignature, MultiSigner,
	SaturatedConversion, 
};
use sp_io::crypto::{sr25519_generate, sr25519_sign};

const SEED: u32 = 2190502;

benchmarks! {
	authorize {
		let owner: AccountOf<T> = account("owner", 100, SEED);
		let operator: AccountOf<T> = account("operator", 100, SEED);
	}: _(RawOrigin::Signed(owner.clone()), operator.clone())
	verify {
		assert!(<AuthorityList<T>>::contains_key(&owner));
		let acc_list = <AuthorityList<T>>::get(&owner);
		assert!(acc_list.contains(&operator));
	}

	cancel_authorize {
		let owner: AccountOf<T> = account("owner", 100, SEED);
        let operator: AccountOf<T> = account("operator", 100, SEED);
		let mut operator_list: BoundedVec<AccountOf<T>, T::AuthorLimit> = Default::default();
        operator_list.try_push(operator.clone()).unwrap();
		<AuthorityList<T>>::insert(&owner, operator_list);
	}: _(RawOrigin::Signed(owner.clone()), operator.clone())
	verify {
        let operator_list = <AuthorityList<T>>::get(&owner);
		assert!(!operator_list.contains(&operator));
	}

	register {
		let oss: AccountOf<T> = account("oss", 100, SEED);
		let domain: BoundedVec<u8, ConstU32<50>> = Default::default();
		let peer_id = [0u8; 38];
	}: _(RawOrigin::Signed(oss.clone()), peer_id.clone(), domain.clone())
	verify {
		assert!(<Oss<T>>::contains_key(&oss));
		let oss_info_right = <Oss<T>>::get(&oss).unwrap();
		let oss_info = OssInfo {
			peer_id: peer_id,
			domain: domain,
		};
		assert_eq!(oss_info_right, oss_info);
	}

	update {
		let oss: AccountOf<T> = account("oss", 100, SEED);
		let peer_id = [5u8; 38];
		let domain: BoundedVec<u8, ConstU32<50>> = Default::default();
		let oss_info = OssInfo {
			peer_id: peer_id.clone(),
			domain: domain.clone(),
		};
		<Oss<T>>::insert(&oss, oss_info);
		let new_peer_id = [6u8; 38];
	}: _(RawOrigin::Signed(oss.clone()), new_peer_id.clone(), domain.clone())
	verify {
		assert!(<Oss<T>>::contains_key(&oss));
		let oss_info = OssInfo {
			peer_id: new_peer_id,
			domain: domain,
		};
		let cur_oss_info = <Oss<T>>::get(&oss).unwrap();
		assert_eq!(cur_oss_info, oss_info);
	}

	destroy {
		let oss: AccountOf<T> = account("oss", 100, SEED);
        let peer_id = [5u8; 38];
		let domain: BoundedVec<u8, ConstU32<50>> = Default::default();
		let oss_info = OssInfo {
			peer_id: peer_id.clone(),
			domain: domain.clone(),
		};
		<Oss<T>>::insert(&oss, oss_info);
	}: _(RawOrigin::Signed(oss.clone()))
	verify {
		assert!(!<Oss<T>>::contains_key(&oss));
	}

	proxy_authorzie {
		let sender = account("origin", 100, SEED);
		let oss: AccountOf<T> = account("oss", 100, SEED);

		let payload = ProxyAuthPayload::<T> {
			oss: oss.clone(),
			exp: 32u32.saturated_into(),
		};

		let mut payload_encode = payload.encode();
		let mut b1 = "<Bytes>".to_string().as_bytes().to_vec();
		let mut b2 = "</Bytes>".to_string().as_bytes().to_vec();

		let mut origin: Vec<u8> = Default::default();
		origin.append(&mut b1);
		origin.append(&mut payload_encode);
		origin.append(&mut b2);

		let caller_public = sr25519_generate(0.into(), None);
		let signature = MultiSignature::Sr25519(sr25519_sign(0.into(), &caller_public, &origin).unwrap());

		let sig = match signature {
			MultiSignature::Sr25519(sig) => sig,
			_ => return Err(frame_benchmarking::BenchmarkError::Stop("asdf")),
		};
		let sig: BoundedVec<u8, ConstU32<64>> = sig.0.to_vec().try_into().map_err(|_| "bounded convert error")?;


	}: _(RawOrigin::Signed(sender), caller_public.clone(), sig, payload)
	verify {
		let account = caller_public.using_encoded(|entropy| {
			AccountOf::<T>::decode(&mut TrailingZeroInput::new(entropy))
				.expect("infinite input; no invalid input; qed")
		});

		let authorty_list = <AuthorityList<T>>::try_get(&account).unwrap();
		assert!(authorty_list.contains(&oss))
	}

	evm_proxy_authorzie {
		let sender = account("origin", 100, SEED);

		let auth_puk = hex::decode("dfa6d7b776c4cac2e9a506e1b9def79c75a53488e3dc320643bb792fbdbe90d1").expect("Valid");
		let auth_puk: sp_core::sr25519::Public = sp_core::sr25519::Public::from_raw(auth_puk.try_into().unwrap());
		let account = auth_puk.using_encoded(|entropy| {
			AccountOf::<T>::decode(&mut TrailingZeroInput::new(entropy))
				.expect("infinite input; no invalid input; qed")
		});

		let oss = hex::decode("02f17b57f801914fd35f95b73b199efe4e37d121f79e39f20ba29f6acd9e355b").expect("Valid");
		let oss: sp_core::sr25519::Public = sp_core::sr25519::Public::from_raw(oss.try_into().unwrap());
		let oss_acc = oss.using_encoded(|entropy| {
			AccountOf::<T>::decode(&mut TrailingZeroInput::new(entropy))
				.expect("infinite input; no invalid input; qed")
		});

		let payload = ProxyAuthPayload::<T> {
			oss: oss_acc.clone(),
			exp: 583561u32.saturated_into(),
		};

		let sig = hex::decode("65665a76bd886775766c77c8c5b43edb3ebb9b5f9860d4ded786b6537cf0fc4e00a5625d8a83d86c02435c82e70132bdb265a7fdb0fb14f7d724540be58b148b1c").expect("Valid");
		let sig: EIP712Signature = sig.try_into().unwrap();
	}: _(RawOrigin::Signed(sender), auth_puk, sig, payload)
	verify {
		let authorty_list = <AuthorityList<T>>::try_get(&account).unwrap();
		assert!(authorty_list.contains(&oss_acc))
	}
}
