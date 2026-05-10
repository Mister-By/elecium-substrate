#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    pub type ElecId = u32;
    pub type CandidId = u32;
    pub type MerkleRoot = [u8; 32];
    pub type Nullifier = [u8; 32];

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type AdminOrigin: frame_support::traits::EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::storage]
    pub type MerkleRoots<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ElecId,
        MerkleRoot,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type ElectionOpen<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ElecId,
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type NullifierUsed<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ElecId,
        Blake2_128Concat,
        Nullifier,
        bool,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ElectionCreated { idelec: ElecId, root: MerkleRoot },
        ElectionClosed { idelec: ElecId },
        VoteAccepted { idelec: ElecId, nullifier: Nullifier, idcandid: CandidId },
    }

    #[pallet::error]
    pub enum Error<T> {
        ElectionNotFound,
        ElectionIsClosed,
        NullifierAlreadyUsed,
        ElectionAlreadyExists,
        InvalidProof,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Créer une élection et stocker sa racine Merkle.
        /// AdminOrigin uniquement (sudo).
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn create_election(
            origin: OriginFor<T>,
            idelec: ElecId,
            root: MerkleRoot,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            ensure!(
                !MerkleRoots::<T>::contains_key(idelec),
                Error::<T>::ElectionAlreadyExists
            );

            MerkleRoots::<T>::insert(idelec, root);
            ElectionOpen::<T>::insert(idelec, true);

            Self::deposit_event(Event::ElectionCreated { idelec, root });
            Ok(())
        }

        /// Fermer une élection.
        /// AdminOrigin uniquement (sudo).
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(5_000, 0))]
        pub fn close_election(
            origin: OriginFor<T>,
            idelec: ElecId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            ensure!(
                MerkleRoots::<T>::contains_key(idelec),
                Error::<T>::ElectionNotFound
            );

            ElectionOpen::<T>::insert(idelec, false);
            Self::deposit_event(Event::ElectionClosed { idelec });
            Ok(())
        }

        /// Soumettre un vote avec preuve Groth16.
        /// vk_bytes    : VerifyingKey sérialisée (ark-serialize compressed)
        /// proof_bytes : Proof sérialisée (ark-serialize compressed)
        /// inputs_bytes: public inputs sérialisés [nullifier_fr, root_fr, idelec_fr]
        /// nullifier   : hash(secret + idelec) en bytes
        /// idcandid    : identifiant du candidat
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(500_000, 0))]
        pub fn vote(
            origin: OriginFor<T>,
            idelec: ElecId,
            vk_bytes: BoundedVec<u8, ConstU32<2048>>,
            proof_bytes: BoundedVec<u8, ConstU32<512>>,
            inputs_bytes: BoundedVec<u8, ConstU32<256>>,
            nullifier: Nullifier,
            idcandid: CandidId,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Vérifier que l'élection existe
            ensure!(
                MerkleRoots::<T>::contains_key(idelec),
                Error::<T>::ElectionNotFound
            );

            // Vérifier que l'élection est ouverte
            ensure!(
                ElectionOpen::<T>::get(idelec),
                Error::<T>::ElectionIsClosed
            );

            // Vérifier que le nullifier n'a pas déjà été utilisé
            ensure!(
                !NullifierUsed::<T>::get(idelec, nullifier),
                Error::<T>::NullifierAlreadyUsed
            );

            // Vérifier la preuve ZK via host function native
            let valid = elecium_host_functions::zk_verifier::verify_groth16(
                &vk_bytes,
                &proof_bytes,
                &inputs_bytes,
            );

            ensure!(valid, Error::<T>::InvalidProof);

            // Enregistrer le nullifier
            NullifierUsed::<T>::insert(idelec, nullifier, true);

            Self::deposit_event(Event::VoteAccepted { idelec, nullifier, idcandid });
            Ok(())
        }

        /// Nettoyer les nullifiers d'une élection fermée.
        /// AdminOrigin uniquement (sudo).
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn cleanup_election(
            origin: OriginFor<T>,
            idelec: ElecId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            ensure!(
                MerkleRoots::<T>::contains_key(idelec),
                Error::<T>::ElectionNotFound
            );
            ensure!(
                !ElectionOpen::<T>::get(idelec),
                Error::<T>::ElectionIsClosed
            );

            let _ = NullifierUsed::<T>::clear_prefix(idelec, u32::MAX, None);
            MerkleRoots::<T>::remove(idelec);
            ElectionOpen::<T>::remove(idelec);

            Ok(())
        }
    }
}

