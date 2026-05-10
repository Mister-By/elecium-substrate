#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod verifier {
    use ark_bn254::{Bn254, Fr};
    use ark_groth16::{Groth16, Proof, VerifyingKey};
    use ark_serialize::CanonicalDeserialize;

    pub fn verify(vk_bytes: &[u8], proof_bytes: &[u8], inputs_bytes: &[u8]) -> bool {
        let vk = match VerifyingKey::<Bn254>::deserialize_compressed(vk_bytes) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let proof = match Proof::<Bn254>::deserialize_compressed(proof_bytes) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let inputs = match Vec::<Fr>::deserialize_compressed(inputs_bytes) {
            Ok(i) => i,
            Err(_) => return false,
        };
        let pvk = ark_groth16::prepare_verifying_key(&vk);
        Groth16::<Bn254>::verify_proof(&pvk, &proof, &inputs).unwrap_or(false)
    }
}

#[sp_runtime_interface::runtime_interface]
pub trait ZkVerifier {
    fn verify_groth16(
        vk_bytes: &[u8],
        proof_bytes: &[u8],
        inputs_bytes: &[u8],
    ) -> bool {
        #[cfg(feature = "std")]
        {
            crate::verifier::verify(vk_bytes, proof_bytes, inputs_bytes)
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = (vk_bytes, proof_bytes, inputs_bytes);
            false
        }
    }
}
