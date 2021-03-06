use rand;
use rand::Rng;

use crate::blueprints::Blueprints;
use crate::chromosome::EgpChromosome;
use crate::component::Component;

/// Performs the mutation genetic operator in-place
pub fn mutate(blueprints: &Blueprints, chromosome: &mut EgpChromosome) {
    let mut rng = rand::thread_rng();

    if rng.gen_range(0., 1.) < 0.5 {
        mutate_activity(blueprints, chromosome);
    } else {
        let n_regulars: usize = chromosome.regular.iter().map(|group| group.len()).sum();

        if rng.gen_range(0., 1.) < 1. / (n_regulars as f32) {
            mutate_binding_site_output(blueprints, chromosome);
        } else {
            mutate_binding_site(blueprints, chromosome);
        }
    }
}

/// Performs the recombination / crossover genetic operator
///
/// The crossover event is either a transfer from parent_a to parent_b, or a
/// removal from parent_a, with equal probability. This is done for balancing
/// the overall chromosome size in the population.
pub fn recombine(
    blueprints: &Blueprints,
    n_transfer: usize,
    parent_a: &EgpChromosome,
    parent_b: &EgpChromosome,
) -> EgpChromosome {
    let mut rng = rand::thread_rng();

    if rng.gen_range(0., 1.) < 0.5 {
        recombine_remove(blueprints, n_transfer, parent_a)
    } else {
        recombine_transfer(blueprints, n_transfer, parent_a, parent_b)
    }
}

fn recombine_transfer(
    blueprints: &Blueprints,
    n_transfer: usize,
    parent: &EgpChromosome,
    donor: &EgpChromosome,
) -> EgpChromosome {
    let mut child = parent.clone();

    let mut rng = rand::thread_rng();

    let nonempty_group = nonempty_group(blueprints);
    let group_len = donor.regular[nonempty_group].len();

    let skip = rng.gen_range(0, group_len);

    let mut n_transfer = if n_transfer < group_len {
        n_transfer
    } else {
        group_len
    };

    for i in skip..group_len {
        if n_transfer <= 0 {
            break;
        }

        child.regular[nonempty_group].push(donor.regular[nonempty_group][i].clone());

        n_transfer -= 1;
    }

    child
}

fn recombine_remove(
    blueprints: &Blueprints,
    n_remove: usize,
    parent: &EgpChromosome,
) -> EgpChromosome {
    let mut child = parent.clone();

    let mut rng = rand::thread_rng();

    let nonempty_group = nonempty_group(blueprints);
    let group_len = child.regular[nonempty_group].len();

    let mut n_remove = if n_remove < group_len {
        n_remove
    } else {
        group_len
    };

    let skip = rng.gen_range(0, group_len);

    for i in skip..group_len {
        if n_remove == 0 || i < child.regular[nonempty_group].len() {
            break;
        }

        child.regular[nonempty_group].remove(i);

        n_remove -= 1;
    }

    child
}

fn mutate_activity(blueprints: &Blueprints, chromosome: &mut EgpChromosome) {
    let (group, member) = pick_group_and_member(blueprints);

    let new_component = Component::from_blueprint(
        &blueprints.regular[group][member],
        blueprints.total_activities,
    );

    let n_compatible = chromosome.regular[group]
        .iter()
        .filter(|component| component.activity == new_component.activity)
        .count();

    if n_compatible == 0 {
        return;
    }

    let mut rng = rand::thread_rng();

    let to_replace = rng.gen_range(0, n_compatible);

    let mut n_encountered = 0;

    for i in 0..chromosome.regular[group].len() {
        if chromosome.regular[group][i].activity == new_component.activity {
            if to_replace == n_encountered {
                chromosome.regular[group][i].activity = new_component.activity;
                chromosome.regular[group][i].label = new_component.label.clone();
                break;
            }

            n_encountered += 1;
        }
    }
}

fn mutate_binding_site(blueprints: &Blueprints, chromosome: &mut EgpChromosome) {
    let mut rng = rand::thread_rng();

    let nonempty_group = nonempty_group(blueprints);
    let group_len = chromosome.regular[nonempty_group].len();

    let component = &mut chromosome.regular[nonempty_group][rng.gen_range(0, group_len)];

    let binding_site_index = rng.gen_range(0, component.binding_sites.len());
    let dimension = rng.gen_range(0, blueprints.total_activities);

    component.binding_sites[binding_site_index][dimension] = rng.gen::<f32>();
}

fn mutate_binding_site_output(blueprints: &Blueprints, chromosome: &mut EgpChromosome) {
    let mut rng = rand::thread_rng();

    let component = &mut chromosome.output;

    let binding_site_index = rng.gen_range(0, component.binding_sites.len());
    let dimension = rng.gen_range(0, blueprints.total_activities);

    component.binding_sites[binding_site_index][dimension] = rng.gen::<f32>();
}

fn nonempty_group(blueprints: &Blueprints) -> usize {
    let mut rng = rand::thread_rng();
    let nonempty_groups = nonempty_groups(blueprints);
    nonempty_groups[rng.gen_range(0, nonempty_groups.len())]
}

fn nonempty_groups(blueprints: &Blueprints) -> Vec<usize> {
    blueprints
        .regular
        .iter()
        .enumerate()
        .filter(|(_group_index, group)| group.len() > 0)
        .map(|(group_index, _group)| group_index)
        .collect()
}

fn pick_group_and_member(blueprints: &Blueprints) -> (usize, usize) {
    let mut rng = rand::thread_rng();

    let group = nonempty_group(blueprints);
    let member = rng.gen_range(0, blueprints.regular[group].len());

    (group, member)
}
