use crate::bitcoin::OutPoint;

#[derive(Debug, Clone)]
pub struct SetupParameters {
    pub alpha: Grin,
    pub beta: Bitcoin,
}

#[derive(Debug, Clone)]
pub struct Grin {
    pub amount: u64,
    pub fee: u64, // for simplicity we don't model separate fee values for different transactions
    pub expiry: u64, // block height
}

#[derive(Debug, Clone)]
pub struct Bitcoin {
    pub asset: u64,
    pub fee: u64,
    pub expiry: u32, // absolute timestamp
    pub inputs: Vec<(OutPoint, u64)>,
    pub change: (bitcoin::Address, u64),
    pub refund_address: bitcoin::Address,
    pub redeem_address: bitcoin::Address,
}

impl Bitcoin {
    pub fn new(
        asset: u64,
        fee: u64,
        expiry: u32,
        inputs: Vec<(OutPoint, u64)>,
        change_address: bitcoin::Address,
        refund_address: bitcoin::Address,
        redeem_address: bitcoin::Address,
    ) -> Result<Bitcoin, ()> {
        let total_input_amount: u64 = inputs.iter().map(|(_, amount)| amount).sum();
        // TODO: use check_operation everywhere
        let change_amount = total_input_amount
            .checked_sub(asset + (2 * fee))
            .ok_or(())?;

        Ok(Bitcoin {
            asset,
            fee,
            expiry,
            inputs,
            change: (change_address, change_amount),
            refund_address,
            redeem_address,
        })
    }

    pub fn fund_output_amount(&self) -> u64 {
        self.asset + self.fee
    }

    pub fn redeem_output_amount(&self) -> u64 {
        self.asset
    }
}
