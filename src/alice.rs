use crate::{
    bitcoin,
    commit::{Commitment, Opening},
    grin, keypair,
    messages::{Message0, Message1, Message2, Message3, Message4},
    setup_parameters::{self, SetupParameters},
};

// TODO: Figure out what to do with bulletproof keys, if anything. For now,
// ignore them since we don't know how we are gonna tackle them
pub struct Alice0 {
    init: SetupParameters,
    secret_grin_init: setup_parameters::GrinFunderSecret,
    SKs_alpha: grin::SKs,
    SKs_beta: bitcoin::SKs,
    y: keypair::KeyPair,
}

impl Alice0 {
    pub fn new(
        init: SetupParameters,
        secret_grin_init: setup_parameters::GrinFunderSecret,
    ) -> (Self, Message0) {
        let SKs_alpha = grin::SKs::keygen();
        let SKs_beta = bitcoin::SKs::keygen();
        let y = keypair::KeyPair::from_slice(b"yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy");

        let commitment = Commitment::commit(&SKs_alpha.public(), &SKs_beta.public(), &y.public_key);

        let state = Alice0 {
            init,
            secret_grin_init,
            SKs_alpha,
            SKs_beta,
            y,
        };

        let message = Message0(commitment);

        (state, message)
    }

    pub fn receive(self, message1: Message1) -> Result<(Alice1, Message2), ()> {
        let opening = Opening::new(
            self.SKs_alpha.public(),
            self.SKs_beta.public(),
            self.y.public_key,
        );

        let beta_redeemer_sigs =
            bitcoin::sign::redeemer(&self.init.beta, &self.SKs_beta, &message1.PKs_beta);

        let state = Alice1 {
            init: self.init,
            secret_grin_init: self.secret_grin_init,
            SKs_alpha: self.SKs_alpha,
            SKs_beta: self.SKs_beta,
            bob_PKs_alpha: message1.PKs_alpha,
            bob_PKs_beta: message1.PKs_beta,
            y: self.y,
        };

        let message = Message2 {
            opening,
            beta_redeemer_sigs,
        };

        Ok((state, message))
    }
}

pub struct Alice1 {
    init: SetupParameters,
    secret_grin_init: setup_parameters::GrinFunderSecret,
    SKs_alpha: grin::SKs,
    SKs_beta: bitcoin::SKs,
    bob_PKs_alpha: grin::PKs,
    bob_PKs_beta: bitcoin::PKs,
    y: keypair::KeyPair,
}

impl Alice1 {
    pub fn receive(self, message: Message3) -> Result<(Alice2, Message4), ()> {
        let (alpha_actions, alpha_redeem_encsig) = grin::sign::funder(
            &self.init.alpha,
            &self.secret_grin_init,
            &self.SKs_alpha,
            &self.bob_PKs_alpha,
            &self.y.public_key,
            message.alpha_redeemer_sigs,
        )
        .map_err(|e| {
            println!("Grin signature verification failed: {:?}", e);
            ()
        })?;

        let beta_encrypted_redeem_action = bitcoin::action::EncryptedRedeem::new(
            &self.init.beta,
            &self.SKs_beta,
            &self.bob_PKs_beta,
            message.beta_redeem_encsig,
        );
        let beta_redeem_action = beta_encrypted_redeem_action.decrypt(&self.y);

        let state = Alice2 {
            alpha_fund_action: alpha_actions.fund,
            alpha_refund_action: alpha_actions.refund,
            beta_redeem_action,
        };

        let message = Message4 {
            alpha_redeem_encsig,
        };

        Ok((state, message))
    }
}

pub struct Alice2 {
    alpha_fund_action: grin::action::Fund,
    alpha_refund_action: grin::action::Refund,
    beta_redeem_action: bitcoin::action::Redeem,
}
