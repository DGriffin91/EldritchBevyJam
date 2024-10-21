use std::borrow::Cow;

use bevy::prelude::*;
pub const FRAC_1_TAU: f32 = 0.15915494309;

pub fn all_children<F: FnMut(Entity)>(
    children: &Children,
    children_query: &Query<&Children>,
    closure: &mut F,
) {
    for child in children {
        if let Ok(children) = children_query.get(*child) {
            all_children(children, children_query, closure);
        }
        closure(*child);
    }
}

#[derive(Component)]
/// Propagate T to all children that have component C, don't forget to add the generic system! propagate::<T, C>
pub struct Propagate<T: Component + Clone>(pub T);

pub fn propagate<T: Component + Clone, C: Component>(
    mut commands: Commands,
    mut entities: Query<(Entity, &mut Propagate<T>)>,
    children_query: Query<&Children>,
    has_needle_component: Query<&C>,
) {
    for (entity, p) in &mut entities {
        let mut found = false;
        if let Ok(children) = children_query.get(entity) {
            all_children(children, &children_query, &mut |entity| {
                if has_needle_component.get(entity).is_ok() {
                    commands.entity(entity).insert(p.0.clone());
                    found = true;
                }
            });
        }
        if found {
            // Seems like this is removed prematurely without found check
            commands.entity(entity).remove::<Propagate<T>>();
        }
    }
}

#[derive(Component)]
/// Propagate T to all children that have component C, don't forget to add the generic system! propagate_default::<T, C>
pub struct PropagateDefault<T: Component + Default>(pub T);

pub fn propagate_default<T: Component + Default, C: Component>(
    mut commands: Commands,
    mut entities: Query<Entity, With<PropagateDefault<T>>>,
    children_query: Query<&Children>,
    has_needle_component: Query<&C>,
) {
    for entity in &mut entities {
        let mut found = false;
        if let Ok(children) = children_query.get(entity) {
            all_children(children, &children_query, &mut |entity| {
                if has_needle_component.get(entity).is_ok() {
                    commands.entity(entity).insert(T::default());
                    found = true;
                }
            });
        }
        if found {
            // Seems like this is removed prematurely without found check
            commands.entity(entity).remove::<PropagateDefault<T>>();
        }
    }
}

#[derive(Component)]
/// Propagate T to all children with Name containing str, don't forget to add the generic system! propagate_to_name::<T>
pub struct PropagateToName<T: Component + Clone>(pub T, pub Cow<'static, str>);

pub fn propagate_to_name<T: Component + Clone>(
    mut commands: Commands,
    mut entities: Query<(Entity, &mut PropagateToName<T>)>,
    children_query: Query<&Children>,
    names: Query<&Name>,
) {
    for (entity, p) in &mut entities {
        if let Ok(children) = children_query.get(entity) {
            let mut found = false;
            all_children(children, &children_query, &mut |entity| {
                if let Ok(name) = names.get(entity) {
                    if name.as_str().contains(&*p.1) {
                        commands.entity(entity).insert(p.0.clone());
                        found = true;
                    }
                }
            });
            if found {
                // Seems like this is removed prematurely without found check
                commands.entity(entity).remove::<PropagateToName<T>>();
            }
        }
    }
}

// like .rem_euclid(1.0)
#[inline(always)]
pub fn pfract(x: f32) -> f32 {
    let y = x.fract();
    if y < 0.0 {
        y + 1.0
    } else {
        y
    }
}
