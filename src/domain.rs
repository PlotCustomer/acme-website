use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The three product families the catalog is organised around.
///
/// They are the specialisations of the modelled `Product` business object:
/// Everyday Goods, Explosives & Weapons, and Vehicles & Gadgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductType {
    EverydayGoods,
    ExplosivesWeapons,
    VehiclesGadgets,
}

impl ProductType {
    pub const ALL: [ProductType; 3] = [
        ProductType::EverydayGoods,
        ProductType::ExplosivesWeapons,
        ProductType::VehiclesGadgets,
    ];

    pub const fn slug(self) -> &'static str {
        match self {
            ProductType::EverydayGoods => "everyday-goods",
            ProductType::ExplosivesWeapons => "explosives-weapons",
            ProductType::VehiclesGadgets => "vehicles-gadgets",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            ProductType::EverydayGoods => "Everyday Goods",
            ProductType::ExplosivesWeapons => "Explosives & Weapons",
            ProductType::VehiclesGadgets => "Vehicles & Gadgets",
        }
    }

    pub const fn icon_path(self) -> &'static str {
        match self {
            ProductType::EverydayGoods => {
                "M6 7V6a6 6 0 1 1 12 0v1h2l1 14H3L4 7h2Zm2 0h8V6a4 4 0 0 0-8 0v1Z"
            }
            ProductType::ExplosivesWeapons => {
                "M12 2 1 21h22L12 2Zm0 5 1 8h-2l1-8Zm0 11a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3Z"
            }
            ProductType::VehiclesGadgets => {
                "M3 5h11v10h2.5l3.5 3.5V17h1v3h-2a3 3 0 0 1-6 0H9a3 3 0 0 1-6 0H1V5h2Zm13 3v5h3.1l-2.1-2.1V8h-1ZM5 17a1 1 0 1 0 2 0 1 1 0 0 0-2 0Zm12 0a1 1 0 1 0 2 0 1 1 0 0 0-2 0Z"
            }
        }
    }

    pub fn from_slug(value: &str) -> Option<Self> {
        match value.trim() {
            "everyday-goods" => Some(ProductType::EverydayGoods),
            "explosives-weapons" => Some(ProductType::ExplosivesWeapons),
            "vehicles-gadgets" => Some(ProductType::VehiclesGadgets),
            _ => None,
        }
    }
}

impl fmt::Display for ProductType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for ProductType {
    type Err = UnknownProductType;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_slug(value).ok_or_else(|| UnknownProductType(value.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown product category: {0}")]
pub struct UnknownProductType(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_category_round_trips_through_its_slug() {
        for product_type in ProductType::ALL {
            assert_eq!(
                ProductType::from_slug(product_type.slug()),
                Some(product_type)
            );
        }
    }

    #[test]
    fn categories_keep_their_modelled_names() {
        assert_eq!(ProductType::EverydayGoods.label(), "Everyday Goods");
        assert_eq!(
            ProductType::ExplosivesWeapons.label(),
            "Explosives & Weapons"
        );
        assert_eq!(ProductType::VehiclesGadgets.label(), "Vehicles & Gadgets");
    }

    #[test]
    fn an_unknown_slug_is_rejected() {
        assert!(ProductType::from_slug("not-a-category").is_none());
    }
}
