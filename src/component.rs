use rand::Rng;
use serde::{Deserialize, Serialize};
use std::iter;

use crate::blueprints::Blueprint;
use crate::vectors;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub activity: usize,
    pub label: String,
    pub binding_sites: Vec<Vec<f32>>, // express new components
    pub binding_sites_groups: Vec<usize>,
    pub weak_binding_sites: Vec<Vec<f32>>, // must connect to already-expressed components
    pub weak_binding_sites_groups: Vec<usize>,
}

impl Component {
    fn activity_vector(&self, total_activities: usize) -> Vec<f32> {
        let mut result = vec![0.; total_activities];
        result[self.activity] = 1.;
        result
    }

    pub fn from_blueprint(blueprint: &Blueprint, total_activities: usize) -> Component {
        let binding_sites_groups = blueprint.binding_sites.clone();
        let binding_sites = random_binding_sites(&blueprint.binding_sites, total_activities);
        let weak_binding_sites_groups = blueprint.weak_binding_sites.clone();
        let weak_binding_sites =
            random_binding_sites(&blueprint.weak_binding_sites, total_activities);

        Component {
            activity: blueprint.activity,
            label: blueprint.label.clone(),
            binding_sites,
            binding_sites_groups,
            weak_binding_sites,
            weak_binding_sites_groups,
        }
    }

    pub fn profile(&self, total_activities: usize, input_bias: f32) -> Vec<f32> {
        let activity_vector = self.activity_vector(total_activities);

        let binding_sites_both: Vec<Vec<f32>> = self
            .binding_sites
            .iter()
            .cloned()
            .chain(self.weak_binding_sites.iter().cloned())
            .collect();

        if binding_sites_both.is_empty() {
            activity_vector
        } else {
            let activity_scaled = vectors::scale(&activity_vector, 1. - input_bias);
            let bindings_average = vectors::average(&binding_sites_both);
            let bindings_scaled = vectors::scale(&bindings_average, input_bias);
            vectors::sum(&activity_scaled, &bindings_scaled)
        }
    }
}

fn random_binding_sites(from: &Vec<usize>, total_activities: usize) -> Vec<Vec<f32>> {
    let mut rng = rand::thread_rng();

    from.iter()
        .map(|_| {
            iter::repeat_with(|| rng.gen::<f32>())
                .take(total_activities)
                .collect()
        })
        .collect()
}
