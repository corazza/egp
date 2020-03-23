use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;

use crate::blueprints::Blueprints;
use crate::chromosome::EgpChromosome;
use crate::component::Component;

pub mod blueprints;
pub mod chromosome;
pub mod component;
pub mod operators;
pub mod vectors;

pub type Phenotype = DiGraph<Expressed, Binding>;

pub enum ComponentIndex {
    Output,
    Regular(usize, usize), // group, index
    Terminal(usize, usize),
}

pub struct Expressed {
    pub label: String, // TODO PERF remove
    pub activity: usize,
    pub index: ComponentIndex,
}

impl Expressed {
    pub fn from_component(component: &Component, index: ComponentIndex) -> Expressed {
        Expressed {
            label: component.label.clone(),
            activity: component.activity,
            index,
        }
    }
}

impl fmt::Debug for Expressed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

#[derive(Copy, Clone)]
pub enum Binding {
    Strong(usize), // index
    Weak(usize),
}

impl Binding {
    pub fn decrement_strong(self) -> Binding {
        match self {
            Binding::Strong(index) => Binding::Strong(index - 1),
            _ => self,
        }
    }
}

impl fmt::Debug for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Binding::Strong(index) => write!(f, "{}", index),
            Binding::Weak(index) => write!(f, "{} (weak)", index),
        }
    }
}

fn get_component<'a>(
    blueprints: &'a Blueprints,
    chromosome: &'a EgpChromosome,
    phenotype: &Phenotype,
    node: NodeIndex,
) -> &'a Component {
    match phenotype[node].index {
        ComponentIndex::Output => &chromosome.output,
        ComponentIndex::Regular(group, index) => &chromosome.regular[group][index],
        ComponentIndex::Terminal(group, index) => &blueprints.terminal[group][index],
    }
}

fn find_min_satisfying_distance<'a, F, I>(
    blueprints: &Blueprints,
    input_bias: f32,
    binding: &Vec<f32>,
    group: I,
    criteria: F,
) -> Option<(usize, f32)>
where
    F: Fn(usize) -> bool,
    I: IntoIterator<Item = &'a Component>,
{
    let mut group = group.into_iter().peekable();

    if group.peek().is_none() {
        return None;
    }

    let first_profile = group
        .peek()
        .unwrap()
        .profile(blueprints.total_activities, input_bias);
    let mut min_distance_index = 0;
    let mut min_distance = vectors::distance(&binding, &first_profile);
    let mut found = false;

    for (index, potential_component) in group.enumerate() {
        let profile = potential_component.profile(blueprints.total_activities, input_bias);
        let distance = vectors::distance(&binding, &profile);

        if (distance <= min_distance) && criteria(index) {
            min_distance = distance;
            min_distance_index = index;
            found = true;
        }
    }

    if found {
        Some((min_distance_index, min_distance))
    } else {
        None
    }
}

fn satisfy_weak(
    blueprints: &Blueprints,
    chromosome: &EgpChromosome,
    phenotype: &mut Phenotype,
    node: NodeIndex,
    offering: &Vec<NodeIndex>,
    input_bias: f32,
) {
    let component = get_component(blueprints, chromosome, &phenotype, node);

    for (binding_index, (binding, _group)) in component
        .weak_binding_sites
        .iter()
        .zip(component.weak_binding_sites_groups.iter())
        .enumerate()
    {
        let offering_components = offering
            .iter()
            .map(|node| get_component(blueprints, chromosome, phenotype, *node));

        let found = find_min_satisfying_distance(
            blueprints,
            input_bias,
            binding,
            offering_components,
            |_| true,
        );

        if let Some((index, _distance)) = found {
            phenotype.add_edge(node, offering[index], Binding::Weak(binding_index));
        }
    }

    // TODO LATER when there are multiple offering activities
    // let looking_for = blueprints.weak_map[&component.label].clone();
    // for potential in offering
    //     .iter()
    //     .map(|node| get_component(blueprints, chromosome, &phenotype, *node))
    //     .filter(|component| component.label == looking_for)
    // {}
}

// TODO the proper way would probably be to allow unlimited terminal expression
// only after no satifying limited expression can be found, and to include terminals
// in distributions

/// Satisfies the bindings for a new node without children
fn satisfy(
    blueprints: &Blueprints,
    chromosome: &EgpChromosome,
    phenotype: &mut Phenotype,
    node: NodeIndex,
    queue: &mut VecDeque<NodeIndex>,
    weak_looking: &mut HashSet<NodeIndex>, // added for later
    weak_offering: &mut HashSet<NodeIndex>,
    expressed_regulars: &mut HashSet<usize>,
    input_bias: f32,
) {
    let component = get_component(blueprints, chromosome, phenotype, node);

    // println!("activity={}", component.activity);

    if blueprints.weak_map.contains_key(&component.label) {
        weak_looking.insert(node);
    }

    if blueprints
        .weak_map
        .values()
        .any(|val| val == &component.label)
    {
        weak_offering.insert(node);
    }

    for (binding_index, (binding, group)) in component
        .binding_sites
        .iter()
        .zip(component.binding_sites_groups.iter())
        .enumerate()
    {
        let regular_find = find_min_satisfying_distance(
            blueprints,
            input_bias,
            binding,
            chromosome.regular[*group].iter(),
            |index| !expressed_regulars.contains(&index),
        );

        let (terminal_index, terminal_distance) = find_min_satisfying_distance(
            blueprints,
            input_bias,
            binding,
            blueprints.terminal[*group].iter(),
            |_| true,
        )
        .unwrap(); // we can always find a terminal, they can be expressed multiple times

        let child = if let Some((index, distance)) = regular_find {
            if distance <= terminal_distance {
                // add regular
                let child = phenotype.add_node(Expressed::from_component(
                    &chromosome.regular[*group][index],
                    ComponentIndex::Regular(*group, index),
                ));

                expressed_regulars.insert(index);

                child
            } else {
                phenotype.add_node(Expressed::from_component(
                    &blueprints.terminal[*group][terminal_index],
                    ComponentIndex::Terminal(*group, terminal_index),
                ))
            }
        } else {
            phenotype.add_node(Expressed::from_component(
                &blueprints.terminal[*group][terminal_index],
                ComponentIndex::Terminal(*group, terminal_index),
            ))
        };

        phenotype.add_edge(
            node,
            child, // HERE
            Binding::Strong(binding_index),
        );

        queue.push_back(child);
    }
}

/// Given blueprints and a chromosome, constructs a phenotype
pub fn express(blueprints: &Blueprints, chromosome: &EgpChromosome) -> Phenotype {
    let input_bias = 0.5;

    let size_est = 1 + chromosome.regular.len() + blueprints.terminal.len();
    let mut phenotype = Phenotype::with_capacity(size_est, size_est);

    let mut expression_queue: VecDeque<NodeIndex> = VecDeque::new();
    // weak bindings go last
    let mut weak_looking: HashSet<NodeIndex> = HashSet::new();
    let mut weak_offering: HashSet<NodeIndex> = HashSet::new();

    let output_node = phenotype.add_node(Expressed::from_component(
        &chromosome.output,
        ComponentIndex::Output,
    ));
    expression_queue.push_back(output_node);

    let mut expressed_regulars: HashSet<usize> = HashSet::new();

    'expression: loop {
        match expression_queue.pop_front() {
            None => break 'expression,
            Some(node) => satisfy(
                blueprints,
                chromosome,
                &mut phenotype,
                node,
                &mut expression_queue,
                &mut weak_looking,
                &mut weak_offering,
                &mut expressed_regulars,
                input_bias,
            ),
        }
    }

    // ensures identical outputs
    let mut offering_vec: Vec<NodeIndex> = weak_offering.into_iter().collect();
    offering_vec.sort();
    let mut weak_looking: Vec<NodeIndex> = weak_looking.into_iter().collect();
    weak_looking.sort();

    for node in weak_looking {
        satisfy_weak(
            blueprints,
            chromosome,
            &mut phenotype,
            node,
            &offering_vec,
            input_bias,
        );
    }

    phenotype
}
