use bevy::prelude::*;

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
pub struct Propagate<T: Component + Clone>(pub T);

pub fn propagate<T: Component + Clone>(
    mut commands: Commands,
    mut entities: Query<(Entity, &mut Propagate<T>)>,
    children_query: Query<&Children>,
) {
    for (entity, p) in &mut entities {
        if let Ok(children) = children_query.get(entity) {
            all_children(children, &children_query, &mut |entity| {
                commands.entity(entity).insert(p.0.clone());
            });
        }
    }
}
