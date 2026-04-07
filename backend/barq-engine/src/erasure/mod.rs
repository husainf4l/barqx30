//! Erasure coding for 11-nines reliability
//!
//! Implements Reed-Solomon erasure coding with SIMD acceleration
//! to achieve data durability beyond AWS S3.

use crate::config::Config;
use anyhow::Result;
use reed_solomon_erasure::galois_8::ReedSolomon;
use tracing::{debug, info};

/// Erasure coding manager
#[allow(dead_code)]
pub struct ErasureCoder {
    data_shards: usize,
    parity_shards: usize,
    encoder: ReedSolomon,
}

#[allow(dead_code)]
impl ErasureCoder {
    /// Creates a new erasure coder
    ///
    /// # Arguments
    /// * `config` - Configuration with data/parity shard counts
    pub fn new(config: &Config) -> Result<Self> {
        let data_shards = config.erasure.data_chunks;
        let parity_shards = config.erasure.parity_chunks;

        let encoder = ReedSolomon::new(data_shards, parity_shards)?;

        info!(
            "Erasure coding initialized: {} data + {} parity shards",
            data_shards, parity_shards
        );

        Ok(Self {
            data_shards,
            parity_shards,
            encoder,
        })
    }

    /// Encodes data into shards with parity
    ///
    /// # Arguments
    /// * `data` - Input data to encode
    ///
    /// # Returns
    /// Vector of shards (data + parity)
    pub fn encode(&self, data: &[u8]) -> Result<Vec<Vec<u8>>> {
        let shard_size = (data.len() + self.data_shards - 1) / self.data_shards;
        let total_shards = self.data_shards + self.parity_shards;

        // Create data shards
        let mut shards: Vec<Vec<u8>> = Vec::with_capacity(total_shards);
        
        for i in 0..self.data_shards {
            let start = i * shard_size;
            let end = std::cmp::min(start + shard_size, data.len());
            
            let mut shard = vec![0u8; shard_size];
            if start < data.len() {
                shard[..end - start].copy_from_slice(&data[start..end]);
            }
            shards.push(shard);
        }

        // Create parity shards
        for _ in 0..self.parity_shards {
            shards.push(vec![0u8; shard_size]);
        }

        // Encode parity
        self.encoder.encode(&mut shards)?;

        debug!(
            "Encoded {} bytes into {} shards ({} data + {} parity)",
            data.len(),
            total_shards,
            self.data_shards,
            self.parity_shards
        );

        Ok(shards)
    }

    /// Decodes shards back into original data
    ///
    /// # Arguments
    /// * `shards` - Vector of shards (some may be None if corrupted)
    /// * `original_size` - Original data size
    ///
    /// # Returns
    /// Reconstructed data
    pub fn decode(&self, mut shards: Vec<Option<Vec<u8>>>, original_size: usize) -> Result<Vec<u8>> {
        // Reconstruct missing shards
        self.encoder.reconstruct(&mut shards)?;

        // Combine data shards
        let mut result = Vec::with_capacity(original_size);
        
        for i in 0..self.data_shards {
            if let Some(ref shard) = shards[i] {
                result.extend_from_slice(shard);
            }
        }

        // Truncate to original size
        result.truncate(original_size);

        debug!("Decoded {} bytes from shards", result.len());
        Ok(result)
    }

    /// Returns the number of shards that can be lost without data loss
    pub fn fault_tolerance(&self) -> usize {
        self.parity_shards
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ErasureConfig;

    #[test]
    fn test_erasure_coding() {
        let config = Config {
            erasure: ErasureConfig {
                data_chunks: 4,
                parity_chunks: 2,
                use_simd: true,
            },
            ..Default::default()
        };

        let coder = ErasureCoder::new(&config).unwrap();

        // Encode
        let data = b"Hello, BARQ X30! This is a test of erasure coding.";
        let shards = coder.encode(data).unwrap();
        assert_eq!(shards.len(), 6); // 4 data + 2 parity

        // Simulate 2 shard failures
        let mut corrupted_shards: Vec<Option<Vec<u8>>> = shards
            .into_iter()
            .enumerate()
            .map(|(i, shard)| if i == 1 || i == 3 { None } else { Some(shard) })
            .collect();

        // Decode
        let recovered = coder.decode(corrupted_shards, data.len()).unwrap();
        assert_eq!(&recovered[..], data);
    }
}
