use super::{CombatStats, Name, SufferDamage, WantsToMelee, Player};
use specs::prelude::*;
use rltk::{console};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_melee, names, combat_stats, mut inflict_damage) = data;

        for (_entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        console::log(&format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        console::log(&format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}
impl DamageSystem {
    pub fn delete_the_dead(ecs : &mut World) {
        let mut dead : Vec<Entity> = Vec::new();
        // Using a scope to make the borrow checker happy
        {
            let combat_stats = ecs.read_storage::<CombatStats>();
            let players = ecs.read_storage::<Player>();
            let entities = ecs.entities();
            for (entity, stats) in (&entities, &combat_stats).join() {
                if stats.hp < 1 {
                    let player = players.get(entity);
                    match player {
                        None => dead.push(entity),
                        Some(_) => console::log("RIP")
                    }
                }
            }
        }
    
        for victim in dead {
            ecs.delete_entity(victim).expect("Unable to delete");
        }
    }
}
