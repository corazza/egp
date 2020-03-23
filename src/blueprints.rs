use std::collections::HashMap;
use std::iter;

// circular imports are for convenience
use crate::chromosome::EgpChromosome;
use crate::component::Component;

pub struct Blueprints {
    pub output: Blueprint, // output always belongs to first group
    pub regular: Vec<Vec<Blueprint>>,
    pub terminal: Vec<Vec<Component>>,
    pub activities_by_group: Vec<usize>, // number of activities in each group (output not counted)
    pub total_activities: usize,
    pub weak_map: HashMap<String, String>,
    pub number_of_regulars: usize,
    pub number_of_terminals: usize,
}

impl Blueprints {
    fn recompute_activities_counter(groups: &mut Vec<Vec<Blueprint>>, counter: &mut usize) {
        for group in groups {
            for blueprint in group {
                blueprint.activity = *counter;
                *counter += 1;
            }
        }
    }

    pub fn recompute_activities(
        regular: &mut Vec<Vec<Blueprint>>,
        terminal: &mut Vec<Vec<Blueprint>>,
    ) {
        let mut activity_n = 0;

        Blueprints::recompute_activities_counter(regular, &mut activity_n);
        Blueprints::recompute_activities_counter(terminal, &mut activity_n);
    }

    pub fn sum_group_lens<T>(groups: &Vec<Vec<T>>) -> usize {
        groups.iter().map(|group| group.len()).sum()
    }

    pub fn from_groups(
        output: Blueprint,
        regular: Vec<Vec<Blueprint>>,
        terminal: Vec<Vec<Blueprint>>,
        weak_map: HashMap<String, String>,
    ) -> Blueprints {
        let activities_by_group: Vec<usize> = regular
            .iter()
            .zip(terminal.iter())
            .map(|(a, b)| a.len() + b.len())
            .collect();

        let total_activities = activities_by_group.iter().sum();

        let terminal: Vec<Vec<Component>> = terminal
            .iter()
            .map(|bps| {
                EgpChromosome::make_group(
                    bps,
                    &iter::repeat(1).take(bps.len()).collect(),
                    total_activities,
                )
            })
            .collect();

        let number_of_regulars = Blueprints::sum_group_lens(&regular);
        let number_of_terminals = Blueprints::sum_group_lens(&terminal);

        Blueprints {
            output,
            regular,
            terminal,
            activities_by_group,
            total_activities,
            weak_map,
            number_of_regulars,
            number_of_terminals,
        }
    }
}

// activity is computed from index in group
// group membership is positional (in Vec<Vec<Blueprint>>)
pub struct Blueprint {
    pub activity: usize,
    pub label: String,
    /// Components with no binding_sites (and no weak binding sites) are terminals determined only
    /// by their activity, not recorded in the chromosome, and therefore expressed multiple times.
    pub binding_sites: Vec<usize>, // group indices
    /// Weak bindings are left as stubs for post-processing, this will build a tree
    pub weak_binding_sites: Vec<usize>, // group indices
}

impl Blueprint {
    pub fn terminal(label: &str) -> Blueprint {
        Blueprint {
            activity: 0,
            label: String::from(label),
            binding_sites: vec![],
            weak_binding_sites: vec![],
        }
    }

    pub fn single_main(label: &str) -> Blueprint {
        Blueprint {
            activity: 0,
            label: String::from(label),
            binding_sites: vec![0],
            weak_binding_sites: vec![],
        }
    }

    pub fn double_main(label: &str) -> Blueprint {
        Blueprint {
            activity: 0,
            label: String::from(label),
            binding_sites: vec![0, 0],
            weak_binding_sites: vec![],
        }
    }

    pub fn terminals(labels: &[&str]) -> Vec<Blueprint> {
        labels
            .iter()
            .map(|label| Blueprint::terminal(label))
            .collect()
    }
}
