//! CTF (Conditional Token Framework) Encoder
//!
//! This module provides functions for encoding CTF contract calls
//! used in Polymarket's prediction markets.

/// Encoder for CTF contract function calls
pub struct CtfEncoder;

impl CtfEncoder {
    /// Encode a redeemPositions call
    ///
    /// # Arguments
    /// * `collateral_token` - The collateral token address (USDC)
    /// * `condition_id` - The condition ID of the market
    /// * `index_sets` - The index sets to redeem (typically [1, 2] for YES/NO)
    ///
    /// # Returns
    /// Hex-encoded function call data
    pub fn encode_redeem_positions(
        collateral_token: &str,
        condition_id: &str,
        index_sets: Vec<u32>,
    ) -> String {
        // redeemPositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] indexSets)
        // Function selector: keccak256("redeemPositions(address,bytes32,bytes32,uint256[])")[0:4] = 0x01b7037c
        let selector = "01b7037c";

        let mut data = String::from("0x");
        data.push_str(selector);

        // Encode collateralToken (address, padded to 32 bytes)
        data.push_str(&encode_address(collateral_token));

        // Encode parentCollectionId (bytes32, all zeros for root)
        data.push_str(&"0".repeat(64));

        // Encode conditionId (bytes32)
        data.push_str(&encode_bytes32(condition_id));

        // Encode indexSets (uint256[] - dynamic array)
        // Offset to array data (4 * 32 = 128 bytes from start of params = 0x80)
        data.push_str(&encode_uint256(128));

        // Array length
        data.push_str(&encode_uint256(index_sets.len() as u64));

        // Array elements
        for index_set in index_sets {
            data.push_str(&encode_uint256(index_set as u64));
        }

        data
    }

    /// Encode a splitPosition call
    ///
    /// # Arguments
    /// * `collateral_token` - The collateral token address (USDC)
    /// * `condition_id` - The condition ID of the market
    /// * `amount` - Amount to split (in smallest units)
    ///
    /// # Returns
    /// Hex-encoded function call data
    pub fn encode_split_position(
        collateral_token: &str,
        condition_id: &str,
        amount: &str,
    ) -> String {
        // splitPosition(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] partition, uint256 amount)
        // Function selector: 0x72ce4275
        let selector = "72ce4275";

        let mut data = String::from("0x");
        data.push_str(selector);

        // Encode collateralToken
        data.push_str(&encode_address(collateral_token));

        // Encode parentCollectionId (all zeros)
        data.push_str(&"0".repeat(64));

        // Encode conditionId
        data.push_str(&encode_bytes32(condition_id));

        // Encode partition offset (5 * 32 = 160 = 0xa0)
        data.push_str(&encode_uint256(160));

        // Encode amount
        data.push_str(&encode_uint256_from_str(amount));

        // Partition array - [1, 2] for binary markets
        data.push_str(&encode_uint256(2)); // array length
        data.push_str(&encode_uint256(1)); // index set 1 (YES)
        data.push_str(&encode_uint256(2)); // index set 2 (NO)

        data
    }

    /// Encode a mergePositions call
    ///
    /// # Arguments
    /// * `collateral_token` - The collateral token address (USDC)
    /// * `condition_id` - The condition ID of the market
    /// * `amount` - Amount to merge (in smallest units)
    ///
    /// # Returns
    /// Hex-encoded function call data
    pub fn encode_merge_positions(
        collateral_token: &str,
        condition_id: &str,
        amount: &str,
    ) -> String {
        // mergePositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] partition, uint256 amount)
        // Function selector: 0xd4e59c76
        let selector = "d4e59c76";

        let mut data = String::from("0x");
        data.push_str(selector);

        // Encode collateralToken
        data.push_str(&encode_address(collateral_token));

        // Encode parentCollectionId (all zeros)
        data.push_str(&"0".repeat(64));

        // Encode conditionId
        data.push_str(&encode_bytes32(condition_id));

        // Encode partition offset (5 * 32 = 160 = 0xa0)
        data.push_str(&encode_uint256(160));

        // Encode amount
        data.push_str(&encode_uint256_from_str(amount));

        // Partition array - [1, 2] for binary markets
        data.push_str(&encode_uint256(2)); // array length
        data.push_str(&encode_uint256(1)); // index set 1 (YES)
        data.push_str(&encode_uint256(2)); // index set 2 (NO)

        data
    }

    /// Encode an ERC20 approve call
    ///
    /// # Arguments
    /// * `spender` - The address to approve
    /// * `amount` - Amount to approve (use u64::MAX for unlimited)
    ///
    /// # Returns
    /// Hex-encoded function call data
    pub fn encode_approve(spender: &str, amount: u128) -> String {
        // approve(address spender, uint256 amount)
        // Function selector: 0x095ea7b3
        let selector = "095ea7b3";

        let mut data = String::from("0x");
        data.push_str(selector);
        data.push_str(&encode_address(spender));
        data.push_str(&encode_uint128(amount));

        data
    }

    /// Encode an ERC20 approve call with maximum amount
    pub fn encode_approve_max(spender: &str) -> String {
        // Use max uint256
        let selector = "095ea7b3";
        let mut data = String::from("0x");
        data.push_str(selector);
        data.push_str(&encode_address(spender));
        // Max uint256
        data.push_str(&"f".repeat(64));
        data
    }
}

// Helper encoding functions

fn encode_address(addr: &str) -> String {
    let addr = addr.trim_start_matches("0x").to_lowercase();
    format!("{:0>64}", addr)
}

fn encode_bytes32(value: &str) -> String {
    let value = value.trim_start_matches("0x").to_lowercase();
    format!("{:0>64}", value)
}

fn encode_uint256(value: u64) -> String {
    format!("{:064x}", value)
}

fn encode_uint128(value: u128) -> String {
    format!("{:064x}", value)
}

fn encode_uint256_from_str(value: &str) -> String {
    let value: u128 = value.parse().unwrap_or(0);
    format!("{:064x}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_redeem_positions() {
        let collateral = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
        let condition_id = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let index_sets = vec![1, 2];

        let result = CtfEncoder::encode_redeem_positions(collateral, condition_id, index_sets);

        // Should start with function selector
        assert!(result.starts_with("0x01b7037c"));
        // Should be a valid hex string
        assert!(result.len() > 10);
    }

    #[test]
    fn test_encode_approve() {
        let spender = "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045";
        let amount = 1000000u128; // 1 USDC

        let result = CtfEncoder::encode_approve(spender, amount);

        assert!(result.starts_with("0x095ea7b3"));
    }

    #[test]
    fn test_encode_split_position() {
        let collateral = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
        let condition_id = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let amount = "1000000";

        let result = CtfEncoder::encode_split_position(collateral, condition_id, amount);

        assert!(result.starts_with("0x72ce4275"));
    }
}
