//! Geographic routing for CDN

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

/// Geographic region
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    UsEast,
    UsWest,
    EuWest,
    EuCentral,
    ApNortheast,
    ApSoutheast,
}

impl Region {
    pub fn as_str(&self) -> &str {
        match self {
            Region::UsEast => "us-east-1",
            Region::UsWest => "us-west-1",
            Region::EuWest => "eu-west-1",
            Region::EuCentral => "eu-central-1",
            Region::ApNortheast => "ap-northeast-1",
            Region::ApSoutheast => "ap-southeast-1",
        }
    }
}

/// Edge node location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeLocation {
    pub node_id: String,
    pub region: Region,
    pub endpoint: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// Geographic router
pub struct GeoRouter {
    locations: HashMap<Region, Vec<EdgeLocation>>,
}

impl GeoRouter {
    /// Create new geo router
    pub fn new() -> Self {
        Self {
            locations: HashMap::new(),
        }
    }

    /// Register edge location
    pub fn register(&mut self, location: EdgeLocation) {
        self.locations
            .entry(location.region.clone())
            .or_insert_with(Vec::new)
            .push(location);
    }

    /// Find nearest edge node for client IP
    pub fn route(&self, _client_ip: IpAddr) -> Option<&EdgeLocation> {
        // Simplified: return first available node
        // In production, use GeoIP database (MaxMind, IP2Location)
        self.locations
            .values()
            .flat_map(|v| v.iter())
            .next()
    }

    /// Get all edge locations
    pub fn all_locations(&self) -> Vec<&EdgeLocation> {
        self.locations
            .values()
            .flat_map(|v| v.iter())
            .collect()
    }

    /// Calculate distance between two points (Haversine formula)
    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371.0; // Earth radius in km
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        
        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos()
            * lat2.to_radians().cos()
            * (d_lon / 2.0).sin().powi(2);
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        r * c
    }
}

impl Default for GeoRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_router() {
        let mut router = GeoRouter::new();
        
        router.register(EdgeLocation {
            node_id: "edge-us-east".to_string(),
            region: Region::UsEast,
            endpoint: "http://edge-us-east.barqcdn.com".to_string(),
            latitude: 40.7128,
            longitude: -74.0060,
        });

        assert!(!router.all_locations().is_empty());
    }
}
