use crate::{
    bitcoin,
    commit::Commitment,
    grin::{self, bulletproof},
    keypair,
    messages::{Message0, Message1, Message2, Message3, Message4},
};
use std::convert::TryInto;

pub struct Bob0<AL, BL> {
    alpha_state: AL,
    beta_state: BL,
    alice_commitment: Commitment,
}

impl Bob0<grin::BobRedeemer0, bitcoin::BobFunder0> {
    pub fn new(
        offer_grin: grin::Offer,
        outputs_grin: grin::SpecialOutputs,
        output_keypairs_grin_redeemer: grin::SpecialOutputKeyPairsRedeemer,
        offer_bitcoin: bitcoin::Offer,
        outputs_bitcoin: bitcoin::WalletOutputs,
        message: Message0,
    ) -> anyhow::Result<(Self, Message1<grin::PKs, bitcoin::PKs>)> {
        let grin_state = grin::bob::BobRedeemer0::new(
            offer_grin,
            outputs_grin,
            output_keypairs_grin_redeemer,
            message.bulletproof_round_1_alice,
        )?;
        let bitcoin_state = bitcoin::bob::BobFunder0::new(offer_bitcoin, outputs_bitcoin);

        Ok((
            Bob0 {
                alpha_state: grin_state.clone(),
                beta_state: bitcoin_state.clone(),
                alice_commitment: message.commitment,
            },
            Message1 {
                PKs_alpha: grin_state.clone().into(),
                PKs_beta: bitcoin_state.into(),
                bulletproof_round_1_bob: grin_state.bulletproof_round_1_self,
            },
        ))
    }

    #[allow(clippy::type_complexity)]
    pub fn receive(
        self,
        Message2 {
            opening,
            beta_redeemer_sigs: alice_bitcoin_refund_signature,
            ..
        }: Message2<bitcoin::Signature>,
    ) -> anyhow::Result<(
        Bob1<grin::BobRedeemer1, bitcoin::BobFunder1>,
        Message3<(grin::RedeemerSigs, grin::bulletproof::Round2), bitcoin::EncryptedSignature>,
    )> {
        let (alice_PKs_grin, alice_PKs_bitcoin, mut Y) = opening.open(self.alice_commitment)?;

        let (grin_state, grin_redeemer_sigs, bulletproof_round_2_self) = self
            .alpha_state
            .transition(alice_PKs_grin.try_into()?, &mut Y)?;
        let (bitcoin_state, bitcoin_redeem_encsig) = self.beta_state.transition(
            alice_PKs_bitcoin.try_into()?,
            alice_bitcoin_refund_signature,
            &Y,
        )?;

        let state = Bob1 {
            alpha_state: grin_state,
            beta_state: bitcoin_state,
            Y,
        };

        let message = Message3 {
            alpha_redeemer_sigs: (grin_redeemer_sigs, bulletproof_round_2_self),
            beta_redeem_encsig: bitcoin_redeem_encsig,
        };

        Ok((state, message))
    }
}

impl Bob0<bitcoin::BobRedeemer0, grin::BobFunder0> {
    pub fn new(
        offer_bitcoin: bitcoin::Offer,
        outputs_bitcoin: bitcoin::WalletOutputs,
        offer_grin: grin::Offer,
        outputs_grin: grin::SpecialOutputs,
        output_keypairs_grin_funder: grin::SpecialOutputKeyPairsFunder,
        message: Message0,
    ) -> anyhow::Result<(Self, Message1<bitcoin::PKs, grin::PKs>)> {
        let bitcoin_state = bitcoin::bob::BobRedeemer0::new(offer_bitcoin, outputs_bitcoin);
        let grin_state = grin::bob::BobFunder0::new(
            offer_grin,
            outputs_grin,
            output_keypairs_grin_funder,
            message.bulletproof_round_1_alice,
        )?;

        Ok((
            Bob0 {
                alpha_state: bitcoin_state.clone(),
                beta_state: grin_state.clone(),
                alice_commitment: message.commitment,
            },
            Message1 {
                PKs_alpha: bitcoin_state.into(),
                PKs_beta: grin_state.clone().into(),
                bulletproof_round_1_bob: grin_state.bulletproof_round_1_self,
            },
        ))
    }

    pub fn receive(
        self,
        Message2 {
            opening,
            beta_redeemer_sigs: alice_grin_redeemer_sigs,
        }: Message2<(grin::RedeemerSigs, bulletproof::Round2)>,
    ) -> anyhow::Result<(
        Bob1<bitcoin::BobRedeemer1, grin::BobFunder1>,
        Message3<bitcoin::Signature, grin::EncryptedSignature>,
    )> {
        let (alice_PKs_bitcoin, alice_PKs_grin, Y) = opening.open(self.alice_commitment)?;

        let (bitcoin_state, bitcoin_redeemer_refund_sig) =
            self.alpha_state.transition(alice_PKs_bitcoin.try_into()?)?;
        let (grin_state, grin_redeem_encsig) = self.beta_state.transition(
            alice_PKs_grin.try_into()?,
            alice_grin_redeemer_sigs.0,
            &Y,
            alice_grin_redeemer_sigs.1,
        )?;

        let state = Bob1 {
            alpha_state: bitcoin_state,
            beta_state: grin_state,
            Y,
        };

        let message = Message3 {
            alpha_redeemer_sigs: bitcoin_redeemer_refund_sig,
            beta_redeem_encsig: grin_redeem_encsig,
        };

        Ok((state, message))
    }
}

pub struct Bob1<AL, BL> {
    alpha_state: AL,
    beta_state: BL,
    Y: keypair::PublicKey,
}

impl Bob1<grin::BobRedeemer1, bitcoin::BobFunder1> {
    pub fn receive(
        self,
        message: Message4<grin::EncryptedSignature>,
    ) -> anyhow::Result<Bob2<grin::BobRedeemer2, bitcoin::BobFunder2>> {
        let grin_state = self
            .alpha_state
            .transition(self.Y, message.alpha_redeem_encsig)?;
        let bitcoin_state = self.beta_state.transition()?;

        Ok(Bob2 {
            alpha_state: grin_state,
            beta_state: bitcoin_state,
        })
    }
}

impl Bob1<bitcoin::BobRedeemer1, grin::BobFunder1> {
    pub fn receive(
        self,
        message: Message4<bitcoin::EncryptedSignature>,
    ) -> anyhow::Result<Bob2<bitcoin::BobRedeemer2, grin::BobFunder2>> {
        let bitcoin_state = self.alpha_state.transition(message.alpha_redeem_encsig)?;
        let grin_state = self.beta_state.transition()?;

        Ok(Bob2 {
            alpha_state: bitcoin_state,
            beta_state: grin_state,
        })
    }
}

pub struct Bob2<AL, BL> {
    pub alpha_state: AL,
    pub beta_state: BL,
}
