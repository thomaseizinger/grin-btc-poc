use crate::keypair::{KeyPair, PublicKey, SecretKey, SECP};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Proof {
    s: SecretKey,
    c: SecretKey,
}

pub fn prove(G: &PublicKey, Gx: &PublicKey, H: &PublicKey, Hx: &PublicKey, x: &SecretKey) -> Proof {
    // NOTE: using thread_rng for PoC and even early stage production but there
    // are more robust ways of doing this which include hashing secret
    // information along with randomness (see https://github.com/bitcoin/bips/pull/893/).
    let r = KeyPair::new_random();

    // Gr
    let mut Gr = *G;
    Gr.mul_assign(&*SECP, &r.secret_key).unwrap();

    // Hr
    let mut Hr = *H;
    Hr.mul_assign(&*SECP, &r.secret_key).unwrap();

    // c = H(G | Gx | H | Hx | Gr | Hr)
    let mut hasher = Sha256::default();
    hasher.input(&G.serialize_vec(&*SECP, true));
    hasher.input(&Gx.serialize_vec(&*SECP, true));
    hasher.input(&H.serialize_vec(&*SECP, true));
    hasher.input(&Hx.serialize_vec(&*SECP, true));
    hasher.input(&Gr.serialize_vec(&*SECP, true));
    hasher.input(&Hr.serialize_vec(&*SECP, true));
    let c = SecretKey::from_slice(&*SECP, &hasher.result()[..]).unwrap();

    // s = r + cx
    let mut s = c.clone();
    s.mul_assign(&*SECP, &x).unwrap();
    s.add_assign(&*SECP, &r.secret_key).unwrap();

    Proof { s, c }
}

pub fn verify(
    G: &PublicKey,
    Gx: &PublicKey,
    H: &PublicKey,
    Hx: &PublicKey,
    proof: &Proof, // (s = r + cx, c)
) -> bool {
    let mut c_neg = proof.c.clone();
    c_neg.neg_assign(&*SECP).unwrap();

    // Gr = Gs + (Gx * -c) = Gr + Gcx - Gcx
    let Gr = {
        let mut Gxc_neg = *Gx;
        // TODO: Don't panic on things controlled by adversary
        Gxc_neg.mul_assign(&*SECP, &c_neg).unwrap();

        let mut Gs = *G;
        Gs.mul_assign(&*SECP, &proof.s).unwrap();
        PublicKey::from_combination(&*SECP, vec![&Gxc_neg, &Gs]).unwrap()
    };

    // Hr = Hs + (Hx * -c) = Hr + Hcx - Hcx
    let Hr = {
        let mut Hxc_neg = *Hx;
        Hxc_neg.mul_assign(&*SECP, &c_neg).unwrap();

        let mut Hs = *H;
        Hs.mul_assign(&*SECP, &proof.s).unwrap();
        PublicKey::from_combination(&*SECP, vec![&Hxc_neg, &Hs]).unwrap()
    };

    // c = H(G | Gx | H | Hx | Gr | Hr)
    let mut hasher = Sha256::default();
    hasher.input(&G.serialize_vec(&*SECP, true));
    hasher.input(&Gx.serialize_vec(&*SECP, true));
    hasher.input(&H.serialize_vec(&*SECP, true));
    hasher.input(&Hx.serialize_vec(&*SECP, true));
    hasher.input(&Gr.serialize_vec(&*SECP, true));
    hasher.input(&Hr.serialize_vec(&*SECP, true));
    let c = SecretKey::from_slice(&*SECP, &hasher.result()[..]).unwrap();

    // c == c'
    proof.c == c
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::keypair::{random_secret_key, G};

    #[test]
    fn prove_and_verify() {
        let x = random_secret_key();
        let mut Gx = *G;
        Gx.mul_assign(&*SECP, &x).unwrap();

        let mut H = *G;
        H.mul_assign(&*SECP, &random_secret_key()).unwrap();

        let mut Hx = H;
        Hx.mul_assign(&*SECP, &x).unwrap();

        let proof = crate::dleq::prove(&*G, &Gx, &H, &Hx, &x);

        assert!(crate::dleq::verify(&*G, &Gx, &H, &Hx, &proof))
    }
}
