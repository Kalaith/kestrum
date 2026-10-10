//! Region views over authored geography and developed city neighborhoods.

use super::CampaignWorld;
use crate::data::{
    economy::Habitation,
    world::{MarkerId, MarkerLocation, Site, SiteId},
};

const CITY_REGION_COUNTRY_RADIUS: f64 = 0.06;
const CITY_REGION_LOCAL_SCALE: f32 = 8.0;
const CITY_REGION_RADIUS: f64 = 0.22;
const CITY_REGION_MIN_RADIUS: f64 = 0.06;
const CITY_REGION_DISTANCE_SCALE: f64 = 0.05;
const CITY_REGION_RADIUS_SPAN: f64 = CITY_REGION_RADIUS - CITY_REGION_MIN_RADIUS;
const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;
const TAU: f64 = std::f64::consts::TAU;

impl CampaignWorld {
    /// Whether a marker opens authored regional geography or a developed city.
    pub fn is_region_available(&self, marker: MarkerId) -> bool {
        let Some(location) = self.marker(marker) else {
            return false;
        };
        match &location.location {
            MarkerLocation::Region { sites, .. } => sites
                .iter()
                .any(|site| self.site(*site).is_some_and(|entry| entry.marker == marker)),
            MarkerLocation::Site { site } => self.city_center(marker, *site).is_some(),
        }
    }

    /// Return the declared members of a legacy region or a city's local vicinity.
    pub fn region_sites(&self, marker: MarkerId) -> Vec<SiteId> {
        let Some(location) = self.marker(marker) else {
            return Vec::new();
        };
        match &location.location {
            MarkerLocation::Region { sites, .. } => sites
                .iter()
                .copied()
                .filter(|site| self.site(*site).is_some_and(|entry| entry.marker == marker))
                .collect(),
            MarkerLocation::Site { site } if self.city_center(marker, *site).is_some() => {
                if self.layout_revision >= 5 {
                    let center = self.site(*site).expect("validated city center");
                    self.sites
                        .iter()
                        .filter(|candidate| {
                            country_distance(center.position, candidate.position)
                                <= CITY_REGION_COUNTRY_RADIUS
                        })
                        .map(|candidate| candidate.id)
                        .collect()
                } else {
                    std::iter::once(*site)
                        .chain(
                            self.adjacent_sites(*site)
                                .into_iter()
                                .filter(|neighbor| neighbor != site),
                        )
                        .collect()
                }
            }
            MarkerLocation::Site { .. } => Vec::new(),
        }
    }

    /// Return a region member's normalized local position.
    ///
    /// Authored regions retain their original coordinates. Revision 5 city
    /// regions use the same affine transform as their authored geography.
    pub fn region_site_position(&self, marker: MarkerId, site: SiteId) -> Option<[f32; 2]> {
        let location = self.marker(marker)?;
        match &location.location {
            MarkerLocation::Region { sites, .. } => {
                if !sites.contains(&site) {
                    return None;
                }
                self.site(site)
                    .filter(|entry| entry.marker == marker)
                    .map(|entry| entry.position)
            }
            MarkerLocation::Site { site: center_id } => {
                let center = self.city_center(marker, *center_id)?;
                let entry = self.site(site)?;
                if self.layout_revision >= 5 {
                    if country_distance(center.position, entry.position)
                        > CITY_REGION_COUNTRY_RADIUS
                    {
                        return None;
                    }
                    self.region_position(marker, entry.position)
                } else if site == *center_id {
                    Some([0.5, 0.5])
                } else if self.adjacent_sites(*center_id).contains(&site) {
                    Some(city_legacy_neighbor_position(center, entry))
                } else {
                    None
                }
            }
        }
    }

    /// Project canonical country coordinates into a revision 5 city vicinity.
    pub fn region_position(&self, marker: MarkerId, country: [f32; 2]) -> Option<[f32; 2]> {
        if self.layout_revision < 5 {
            return None;
        }
        let MarkerLocation::Site { site } = &self.marker(marker)?.location else {
            return None;
        };
        let center = self.city_center(marker, *site)?;
        Some([
            0.5 + (country[0] - center.position[0]) * CITY_REGION_LOCAL_SCALE,
            0.5 + (country[1] - center.position[1]) * CITY_REGION_LOCAL_SCALE,
        ])
    }

    /// Linear scale applied to country-normalized widths in a local city view.
    pub fn region_geometry_scale(&self, marker: MarkerId) -> Option<f32> {
        self.region_position(marker, [0.5, 0.5])
            .map(|_| CITY_REGION_LOCAL_SCALE)
    }

    fn city_center(&self, marker: MarkerId, site: SiteId) -> Option<&Site> {
        let center = self
            .site(site)
            .filter(|center| center.marker == marker && center.habitation >= Habitation::City)?;
        self.development
            .get(&site)
            .filter(|development| !development.ruined)?;
        Some(center)
    }
}

fn city_legacy_neighbor_position(center: &Site, neighbor: &Site) -> [f32; 2] {
    let dx = f64::from(neighbor.position[0]) - f64::from(center.position[0]);
    let dy = f64::from(neighbor.position[1]) - f64::from(center.position[1]);
    let length = dx.hypot(dy);
    let (x, y, radius) = if length.is_finite() && length > 0.0 {
        let radius = CITY_REGION_MIN_RADIUS
            + CITY_REGION_RADIUS_SPAN * (length / (length + CITY_REGION_DISTANCE_SCALE));
        (dx / length, dy / length, radius)
    } else {
        let (x, y) = stable_direction(neighbor.id);
        (x, y, CITY_REGION_MIN_RADIUS)
    };
    [(0.5 + x * radius) as f32, (0.5 + y * radius) as f32]
}

fn stable_direction(site: SiteId) -> (f64, f64) {
    let angle = (f64::from(site.0) * GOLDEN_ANGLE) % TAU;
    (angle.cos(), angle.sin())
}

fn country_distance(a: [f32; 2], b: [f32; 2]) -> f64 {
    let dx = f64::from(a[0]) - f64::from(b[0]);
    let dy = f64::from(a[1]) - f64::from(b[1]);
    dx.hypot(dy)
}
