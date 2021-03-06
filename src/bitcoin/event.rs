use crate::{
    bitcoin::{
        transaction::{fund_transaction, redeem_transaction},
        wallet_outputs::WalletOutputs,
        Offer, PKs,
    },
    keypair::PublicKey,
};
use ::bitcoin::{
    hashes::{sha256d, Hash},
    util::bip143::SighashComponents,
};
use secp256k1zkp::Message;

pub struct Redeem {
    // To identify the redeem transaction on Bitcoin
    pub txid: sha256d::Hash,
    // To extract the correct signature from the witness stack
    pub funder_pk: PublicKey,
    pub message_hash: Message,
}

impl Redeem {
    pub fn new(
        offer: &Offer,
        wallet_outputs: &WalletOutputs,
        redeemer_PKs: &PKs,
        funder_PKs: &PKs,
    ) -> anyhow::Result<Self> {
        let (fund_transaction, fund_output_script) =
            fund_transaction(&offer, &wallet_outputs, &redeemer_PKs.X, &funder_PKs.X)?;
        let redeem_transaction =
            redeem_transaction(&offer, &wallet_outputs, fund_transaction.txid());

        let redeem_digest = SighashComponents::new(&redeem_transaction).sighash_all(
            &redeem_transaction.input[0],
            &fund_output_script,
            fund_transaction.output[0].value,
        );
        let message_hash = Message::from_slice(&redeem_digest.into_inner())?;

        Ok(Self {
            txid: redeem_transaction.txid(),
            funder_pk: funder_PKs.X,
            message_hash,
        })
    }
}
