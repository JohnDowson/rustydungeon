use crate::{Enemy, Map, Name, Position, Viewshed, WantsToMelee};
use rltk::{console, field_of_view, Point};
use specs::prelude::*;

pub struct EnemyAI {}

impl<'a> System<'a> for EnemyAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            mut viewshed,
            enemy,
            name,
            mut position,
            mut wants_to_melee,
            entities,
        ) = data;

        for (mut viewshed, _enemy, name, mut pos, entity) in
            (&mut viewshed, &enemy, &name, &mut position, &entities).join()
        {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                // Attack goes here
                console::log(&format!("{} wants to melee player", name.name));
                wants_to_melee.insert(entity, WantsToMelee{ target: *player_entity }).expect("Unable to insert attack");
            } else if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!("{} pathes to player", name.name));
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map,
                );
                if path.success && path.steps.len() > 1 {
                    let mut idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
