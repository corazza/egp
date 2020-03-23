use rand::Rng;
use serde::{Deserialize, Serialize};
use std::iter;

use crate::blueprints::{Blueprint, Blueprints};
use crate::component::Component;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EgpChromosome {
    pub output: Component,
    pub regular: Vec<Vec<Component>>,
}

impl EgpChromosome {
    pub fn make_many(bp: &Blueprint, n: usize, total_activities: usize) -> Vec<Component> {
        iter::repeat_with(|| Component::from_blueprint(bp, total_activities))
            .take(n)
            .collect()
    }

    pub fn make_group(
        blueprints: &Vec<Blueprint>,
        distribution: &Vec<usize>,
        total_activities: usize,
    ) -> Vec<Component> {
        blueprints
            .iter()
            .zip(distribution.iter())
            .map(|(bp, n)| EgpChromosome::make_many(bp, *n, total_activities))
            .flatten()
            .collect()
    }

    fn distribution(number_of_regulars: usize, size: usize) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let distribution: Vec<f64> = iter::repeat_with(|| rng.gen::<f64>())
            .take(number_of_regulars)
            .collect();
        let sum: f64 = distribution.iter().sum();
        let mut distribution: Vec<usize> = distribution
            .iter()
            .map(|a| (size as f64 * (a / sum)) as usize)
            .collect();
        let missing: usize = size - distribution.iter().sum::<usize>();

        for _ in 0..missing {
            let index = rng.gen_range(0, number_of_regulars);
            distribution[index] += 1;
        }

        assert_eq!(size, distribution.iter().sum::<usize>());

        distribution
    }

    pub fn ancestor_from_blueprints(
        // parameters: &Parameters,
        blueprints: &Blueprints,
        size: usize,
    ) -> EgpChromosome {
        assert!(
            size > blueprints.number_of_terminals + 1,
            "need size > {}",
            blueprints.number_of_terminals + 1
        );

        let regulars_distribution = EgpChromosome::distribution(
            blueprints.number_of_regulars,
            size - 1 - blueprints.number_of_terminals,
        );

        let regular: Vec<Vec<Component>> = blueprints
            .regular
            .iter()
            .map(|bps| {
                EgpChromosome::make_group(bps, &regulars_distribution, blueprints.total_activities)
            })
            .collect();

        let output = Component::from_blueprint(&blueprints.output, blueprints.total_activities);

        EgpChromosome { output, regular }
    }
}
